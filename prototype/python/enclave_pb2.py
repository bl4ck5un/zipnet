# -*- coding: utf-8 -*-
# Generated by the protocol buffer compiler.  DO NOT EDIT!
# source: enclave.proto

from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from google.protobuf import reflection as _reflection
from google.protobuf import symbol_database as _symbol_database
# @@protoc_insertion_point(imports)

_sym_db = _symbol_database.Default()




DESCRIPTOR = _descriptor.FileDescriptor(
  name='enclave.proto',
  package='rpc',
  syntax='proto3',
  serialized_options=None,
  create_key=_descriptor._internal_create_key,
  serialized_pb=b'\n\renclave.proto\x12\x03rpc\"M\n\x0fSchedulingState\x12\r\n\x05round\x18\x01 \x01(\r\x12\x17\n\x0freservation_map\x18\x02 \x03(\x08\x12\x12\n\nfootprints\x18\x03 \x03(\t\"T\n\x11SchedulingRequest\x12\'\n\tcur_state\x18\x01 \x01(\x0b\x32\x14.rpc.SchedulingState\x12\x16\n\x0e\x63ur_dc_message\x18\x02 \x01(\t\"d\n\x12SchedulingResponse\x12\'\n\tnew_state\x18\x01 \x01(\x0b\x32\x14.rpc.SchedulingState\x12\x16\n\x0enew_dc_message\x18\x02 \x01(\t\x12\r\n\x05\x66inal\x18\x03 \x01(\x08\"o\n\x10\x41ggregateRequest\x12\r\n\x05round\x18\x01 \x01(\r\x12\x0f\n\x07user_id\x18\x02 \x01(\x0c\x12\x14\n\x0cuser_message\x18\x03 \x01(\x0c\x12%\n\x0b\x63urrent_agg\x18\x04 \x01(\x0b\x32\x10.rpc.Aggregation\"\\\n\x0b\x41ggregation\x12\x1e\n\x16user_id_in_aggregation\x18\x01 \x03(\x0c\x12 \n\x18\x63urrent_aggregated_value\x18\x02 \x01(\x0c\x12\x0b\n\x03sig\x18\x03 \x01(\x0c\"6\n\x11\x41ggregateResponse\x12!\n\x07new_agg\x18\x01 \x01(\x0b\x32\x10.rpc.Aggregation\"\x07\n\x05\x45mpty2F\n\x07\x65nclave\x12;\n\x08schedule\x12\x16.rpc.SchedulingRequest\x1a\x17.rpc.SchedulingResponseb\x06proto3'
)




_SCHEDULINGSTATE = _descriptor.Descriptor(
  name='SchedulingState',
  full_name='rpc.SchedulingState',
  filename=None,
  file=DESCRIPTOR,
  containing_type=None,
  create_key=_descriptor._internal_create_key,
  fields=[
    _descriptor.FieldDescriptor(
      name='round', full_name='rpc.SchedulingState.round', index=0,
      number=1, type=13, cpp_type=3, label=1,
      has_default_value=False, default_value=0,
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR,  create_key=_descriptor._internal_create_key),
    _descriptor.FieldDescriptor(
      name='reservation_map', full_name='rpc.SchedulingState.reservation_map', index=1,
      number=2, type=8, cpp_type=7, label=3,
      has_default_value=False, default_value=[],
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR,  create_key=_descriptor._internal_create_key),
    _descriptor.FieldDescriptor(
      name='footprints', full_name='rpc.SchedulingState.footprints', index=2,
      number=3, type=9, cpp_type=9, label=3,
      has_default_value=False, default_value=[],
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR,  create_key=_descriptor._internal_create_key),
  ],
  extensions=[
  ],
  nested_types=[],
  enum_types=[
  ],
  serialized_options=None,
  is_extendable=False,
  syntax='proto3',
  extension_ranges=[],
  oneofs=[
  ],
  serialized_start=22,
  serialized_end=99,
)


_SCHEDULINGREQUEST = _descriptor.Descriptor(
  name='SchedulingRequest',
  full_name='rpc.SchedulingRequest',
  filename=None,
  file=DESCRIPTOR,
  containing_type=None,
  create_key=_descriptor._internal_create_key,
  fields=[
    _descriptor.FieldDescriptor(
      name='cur_state', full_name='rpc.SchedulingRequest.cur_state', index=0,
      number=1, type=11, cpp_type=10, label=1,
      has_default_value=False, default_value=None,
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR,  create_key=_descriptor._internal_create_key),
    _descriptor.FieldDescriptor(
      name='cur_dc_message', full_name='rpc.SchedulingRequest.cur_dc_message', index=1,
      number=2, type=9, cpp_type=9, label=1,
      has_default_value=False, default_value=b"".decode('utf-8'),
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR,  create_key=_descriptor._internal_create_key),
  ],
  extensions=[
  ],
  nested_types=[],
  enum_types=[
  ],
  serialized_options=None,
  is_extendable=False,
  syntax='proto3',
  extension_ranges=[],
  oneofs=[
  ],
  serialized_start=101,
  serialized_end=185,
)


_SCHEDULINGRESPONSE = _descriptor.Descriptor(
  name='SchedulingResponse',
  full_name='rpc.SchedulingResponse',
  filename=None,
  file=DESCRIPTOR,
  containing_type=None,
  create_key=_descriptor._internal_create_key,
  fields=[
    _descriptor.FieldDescriptor(
      name='new_state', full_name='rpc.SchedulingResponse.new_state', index=0,
      number=1, type=11, cpp_type=10, label=1,
      has_default_value=False, default_value=None,
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR,  create_key=_descriptor._internal_create_key),
    _descriptor.FieldDescriptor(
      name='new_dc_message', full_name='rpc.SchedulingResponse.new_dc_message', index=1,
      number=2, type=9, cpp_type=9, label=1,
      has_default_value=False, default_value=b"".decode('utf-8'),
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR,  create_key=_descriptor._internal_create_key),
    _descriptor.FieldDescriptor(
      name='final', full_name='rpc.SchedulingResponse.final', index=2,
      number=3, type=8, cpp_type=7, label=1,
      has_default_value=False, default_value=False,
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR,  create_key=_descriptor._internal_create_key),
  ],
  extensions=[
  ],
  nested_types=[],
  enum_types=[
  ],
  serialized_options=None,
  is_extendable=False,
  syntax='proto3',
  extension_ranges=[],
  oneofs=[
  ],
  serialized_start=187,
  serialized_end=287,
)


_AGGREGATEREQUEST = _descriptor.Descriptor(
  name='AggregateRequest',
  full_name='rpc.AggregateRequest',
  filename=None,
  file=DESCRIPTOR,
  containing_type=None,
  create_key=_descriptor._internal_create_key,
  fields=[
    _descriptor.FieldDescriptor(
      name='round', full_name='rpc.AggregateRequest.round', index=0,
      number=1, type=13, cpp_type=3, label=1,
      has_default_value=False, default_value=0,
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR,  create_key=_descriptor._internal_create_key),
    _descriptor.FieldDescriptor(
      name='user_id', full_name='rpc.AggregateRequest.user_id', index=1,
      number=2, type=12, cpp_type=9, label=1,
      has_default_value=False, default_value=b"",
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR,  create_key=_descriptor._internal_create_key),
    _descriptor.FieldDescriptor(
      name='user_message', full_name='rpc.AggregateRequest.user_message', index=2,
      number=3, type=12, cpp_type=9, label=1,
      has_default_value=False, default_value=b"",
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR,  create_key=_descriptor._internal_create_key),
    _descriptor.FieldDescriptor(
      name='current_agg', full_name='rpc.AggregateRequest.current_agg', index=3,
      number=4, type=11, cpp_type=10, label=1,
      has_default_value=False, default_value=None,
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR,  create_key=_descriptor._internal_create_key),
  ],
  extensions=[
  ],
  nested_types=[],
  enum_types=[
  ],
  serialized_options=None,
  is_extendable=False,
  syntax='proto3',
  extension_ranges=[],
  oneofs=[
  ],
  serialized_start=289,
  serialized_end=400,
)


_AGGREGATION = _descriptor.Descriptor(
  name='Aggregation',
  full_name='rpc.Aggregation',
  filename=None,
  file=DESCRIPTOR,
  containing_type=None,
  create_key=_descriptor._internal_create_key,
  fields=[
    _descriptor.FieldDescriptor(
      name='user_id_in_aggregation', full_name='rpc.Aggregation.user_id_in_aggregation', index=0,
      number=1, type=12, cpp_type=9, label=3,
      has_default_value=False, default_value=[],
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR,  create_key=_descriptor._internal_create_key),
    _descriptor.FieldDescriptor(
      name='current_aggregated_value', full_name='rpc.Aggregation.current_aggregated_value', index=1,
      number=2, type=12, cpp_type=9, label=1,
      has_default_value=False, default_value=b"",
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR,  create_key=_descriptor._internal_create_key),
    _descriptor.FieldDescriptor(
      name='sig', full_name='rpc.Aggregation.sig', index=2,
      number=3, type=12, cpp_type=9, label=1,
      has_default_value=False, default_value=b"",
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR,  create_key=_descriptor._internal_create_key),
  ],
  extensions=[
  ],
  nested_types=[],
  enum_types=[
  ],
  serialized_options=None,
  is_extendable=False,
  syntax='proto3',
  extension_ranges=[],
  oneofs=[
  ],
  serialized_start=402,
  serialized_end=494,
)


_AGGREGATERESPONSE = _descriptor.Descriptor(
  name='AggregateResponse',
  full_name='rpc.AggregateResponse',
  filename=None,
  file=DESCRIPTOR,
  containing_type=None,
  create_key=_descriptor._internal_create_key,
  fields=[
    _descriptor.FieldDescriptor(
      name='new_agg', full_name='rpc.AggregateResponse.new_agg', index=0,
      number=1, type=11, cpp_type=10, label=1,
      has_default_value=False, default_value=None,
      message_type=None, enum_type=None, containing_type=None,
      is_extension=False, extension_scope=None,
      serialized_options=None, file=DESCRIPTOR,  create_key=_descriptor._internal_create_key),
  ],
  extensions=[
  ],
  nested_types=[],
  enum_types=[
  ],
  serialized_options=None,
  is_extendable=False,
  syntax='proto3',
  extension_ranges=[],
  oneofs=[
  ],
  serialized_start=496,
  serialized_end=550,
)


_EMPTY = _descriptor.Descriptor(
  name='Empty',
  full_name='rpc.Empty',
  filename=None,
  file=DESCRIPTOR,
  containing_type=None,
  create_key=_descriptor._internal_create_key,
  fields=[
  ],
  extensions=[
  ],
  nested_types=[],
  enum_types=[
  ],
  serialized_options=None,
  is_extendable=False,
  syntax='proto3',
  extension_ranges=[],
  oneofs=[
  ],
  serialized_start=552,
  serialized_end=559,
)

_SCHEDULINGREQUEST.fields_by_name['cur_state'].message_type = _SCHEDULINGSTATE
_SCHEDULINGRESPONSE.fields_by_name['new_state'].message_type = _SCHEDULINGSTATE
_AGGREGATEREQUEST.fields_by_name['current_agg'].message_type = _AGGREGATION
_AGGREGATERESPONSE.fields_by_name['new_agg'].message_type = _AGGREGATION
DESCRIPTOR.message_types_by_name['SchedulingState'] = _SCHEDULINGSTATE
DESCRIPTOR.message_types_by_name['SchedulingRequest'] = _SCHEDULINGREQUEST
DESCRIPTOR.message_types_by_name['SchedulingResponse'] = _SCHEDULINGRESPONSE
DESCRIPTOR.message_types_by_name['AggregateRequest'] = _AGGREGATEREQUEST
DESCRIPTOR.message_types_by_name['Aggregation'] = _AGGREGATION
DESCRIPTOR.message_types_by_name['AggregateResponse'] = _AGGREGATERESPONSE
DESCRIPTOR.message_types_by_name['Empty'] = _EMPTY
_sym_db.RegisterFileDescriptor(DESCRIPTOR)

SchedulingState = _reflection.GeneratedProtocolMessageType('SchedulingState', (_message.Message,), {
  'DESCRIPTOR' : _SCHEDULINGSTATE,
  '__module__' : 'enclave_pb2'
  # @@protoc_insertion_point(class_scope:rpc.SchedulingState)
  })
_sym_db.RegisterMessage(SchedulingState)

SchedulingRequest = _reflection.GeneratedProtocolMessageType('SchedulingRequest', (_message.Message,), {
  'DESCRIPTOR' : _SCHEDULINGREQUEST,
  '__module__' : 'enclave_pb2'
  # @@protoc_insertion_point(class_scope:rpc.SchedulingRequest)
  })
_sym_db.RegisterMessage(SchedulingRequest)

SchedulingResponse = _reflection.GeneratedProtocolMessageType('SchedulingResponse', (_message.Message,), {
  'DESCRIPTOR' : _SCHEDULINGRESPONSE,
  '__module__' : 'enclave_pb2'
  # @@protoc_insertion_point(class_scope:rpc.SchedulingResponse)
  })
_sym_db.RegisterMessage(SchedulingResponse)

AggregateRequest = _reflection.GeneratedProtocolMessageType('AggregateRequest', (_message.Message,), {
  'DESCRIPTOR' : _AGGREGATEREQUEST,
  '__module__' : 'enclave_pb2'
  # @@protoc_insertion_point(class_scope:rpc.AggregateRequest)
  })
_sym_db.RegisterMessage(AggregateRequest)

Aggregation = _reflection.GeneratedProtocolMessageType('Aggregation', (_message.Message,), {
  'DESCRIPTOR' : _AGGREGATION,
  '__module__' : 'enclave_pb2'
  # @@protoc_insertion_point(class_scope:rpc.Aggregation)
  })
_sym_db.RegisterMessage(Aggregation)

AggregateResponse = _reflection.GeneratedProtocolMessageType('AggregateResponse', (_message.Message,), {
  'DESCRIPTOR' : _AGGREGATERESPONSE,
  '__module__' : 'enclave_pb2'
  # @@protoc_insertion_point(class_scope:rpc.AggregateResponse)
  })
_sym_db.RegisterMessage(AggregateResponse)

Empty = _reflection.GeneratedProtocolMessageType('Empty', (_message.Message,), {
  'DESCRIPTOR' : _EMPTY,
  '__module__' : 'enclave_pb2'
  # @@protoc_insertion_point(class_scope:rpc.Empty)
  })
_sym_db.RegisterMessage(Empty)



_ENCLAVE = _descriptor.ServiceDescriptor(
  name='enclave',
  full_name='rpc.enclave',
  file=DESCRIPTOR,
  index=0,
  serialized_options=None,
  create_key=_descriptor._internal_create_key,
  serialized_start=561,
  serialized_end=631,
  methods=[
  _descriptor.MethodDescriptor(
    name='schedule',
    full_name='rpc.enclave.schedule',
    index=0,
    containing_service=None,
    input_type=_SCHEDULINGREQUEST,
    output_type=_SCHEDULINGRESPONSE,
    serialized_options=None,
    create_key=_descriptor._internal_create_key,
  ),
])
_sym_db.RegisterServiceDescriptor(_ENCLAVE)

DESCRIPTOR.services_by_name['enclave'] = _ENCLAVE

# @@protoc_insertion_point(module_scope)
