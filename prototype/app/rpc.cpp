#include "rpc.h"

#include "../common/converter.h"
#include "../common/interface_structs.h"
#include "../common/messages.hpp"
#include "Enclave_u.h"
#include "logging.h"
#include "rpc_types.h"

grpc::Status ecall_failure =
    grpc::Status(grpc::StatusCode::INTERNAL, "ecall failure");

SchedulingState set_state(const rpc::SchedulingState& rpc_sched_state)
{
  SchedulingState state;

  state.round = rpc_sched_state.round();

  const auto& rmap = rpc_sched_state.reservation_map();
  const auto& fps = rpc_sched_state.footprints();
  assert(rmap.size() == N_SLOTS);
  assert(fps.size() == N_SLOTS);
  for (size_t i = 0; i < N_SLOTS; i++) {
    state.reservation.set(i, rmap[i]);
  }

  state.footprints = FootprintsFromString(fps.begin(), fps.end());

  return state;
}

grpc::Status RpcServer::schedule(::grpc::ServerContext* context,
                                 const ::rpc::SchedulingRequest* request,
                                 ::rpc::SchedulingResponse* response)
{
  int ret;

  // build state
  SchedulingState state = set_state(request->cur_state());

  SPDLOG_INFO("state={}", state.to_string());

  SchedulingMessage prev_message;
  if (state.round > 0) {
    // prev_message is not set for the first round
    prev_message.message =
        FootprintsFromString(request->cur_state().footprints().begin(),
                             request->cur_state().footprints().end());
  }

  SchedulingMessage new_message;

  sgx_status_t ecall_status =
      ecall_scheduling(eid, &ret, &prev_message, &state, &new_message);
  if (ecall_status != SGX_SUCCESS) {
    return grpc::Status(grpc::StatusCode::UNKNOWN, "ecall failure");
  }

  if (ret == SCHEDULE_CONTINUE || ret == SCHEDULE_DONE) {
    SPDLOG_INFO(ret == SCHEDULE_CONTINUE ? "continue" : "done");
    SPDLOG_INFO("next round: {}", state.round);
    SPDLOG_INFO("new state: {}", state.to_string());
    SPDLOG_INFO("new message: {}", new_message.to_string());

    // allocate state
    auto* new_st = new rpc::SchedulingState{};
    new_st->set_round(state.round);

    for (size_t i = 0; i < N_SLOTS; i++) {
      // TODO: check this does not mess up the orders
      new_st->add_reservation_map(state.reservation.test(i));
      new_st->add_footprints(state.footprints[i].to_string());
    }

    response->set_allocated_new_state(new_st);

    // build response
    response->set_message_to_broadcast(new_message.to_string());
    response->set_final(ret == SCHEDULE_DONE);
    return grpc::Status::OK;
  }

  SPDLOG_ERROR("sched failed {}", ret);
  return grpc::Status(grpc::StatusCode::UNKNOWN,
                      fmt::format("sched failure {}", ret));
}

grpc::Status RpcServer::aggregate(::grpc::ServerContext* context,
                                  const ::rpc::AggregateRequest* request,
                                  ::rpc::AggregateResponse* response)
{
  try {
    // unmarshal
    AggregatedMessage cur_agg;
    rpc_type_to_enclave_type(cur_agg, request->current_agg());

    UserMessage user_msg;
    rpc_type_to_enclave_type(user_msg, request->msg());

    // marshal
    AggregatedMessage_C cur_agg_bin, new_agg_bin;
    UserMessage_C user_msg_bin;
    cur_agg.marshal(&cur_agg_bin);
    user_msg.marshal(&user_msg_bin);

    int ret;
    sgx_status_t st = ecall_aggregate(
        this->eid, &ret, &user_msg_bin, &cur_agg_bin, &new_agg_bin);
    if (st != SGX_SUCCESS || ret != GOOD) {
      SPDLOG_ERROR("ecall_aggregate failed with {} {}", st, ret);
      return ecall_failure;
    }

    AggregatedMessage new_agg(&new_agg_bin);
    auto* new_agg_rpc = new rpc::Aggregation{};

    SPDLOG_INFO("new agg: {}", new_agg.to_string());

    enclave_type_to_rpc_type(new_agg_rpc, new_agg);
    response->set_allocated_new_agg(new_agg_rpc);

    return grpc::Status::OK;
  } catch (const std::exception& e) {
    SPDLOG_CRITICAL("E: {}", e.what());
    return grpc::Status(grpc::StatusCode::INTERNAL, e.what());
  }
}
