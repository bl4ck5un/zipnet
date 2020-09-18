#ifndef TOWN_CRIER_RPC_H
#define TOWN_CRIER_RPC_H

#include <grpc/grpc.h>
#include <sgx_eid.h>

#include <cstdio>

#include "../common/messages.hpp"
#include "enclave.grpc.pb.h"
#include "enclave.pb.h"

class RpcServer final : public rpc::enclave::Service
{
 private:
  sgx_enclave_id_t eid;

 public:
  explicit RpcServer(sgx_enclave_id_t eid) : eid(eid) {}

  grpc::Status schedule(grpc::ServerContext* context,
                        const rpc::SchedulingRequest* request,
                        rpc::SchedulingResponse* response) override;

  grpc::Status aggregate(::grpc::ServerContext* context,
                         const ::rpc::AggregateRequest* request,
                         ::rpc::AggregateResponse* response) override;
};

SchedulingState set_state(const rpc::SchedulingState&);

#endif  // TOWN_CRIER_RPC_H
