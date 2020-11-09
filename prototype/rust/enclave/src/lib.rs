// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License..

#![no_std]

extern crate sgx_types;
#[macro_use]
extern crate sgx_tstd as std;
#[macro_use]
extern crate sgx_tunittest;
extern crate interface;
extern crate sgx_tcrypto;

#[macro_use]
extern crate quick_error;

extern crate byteorder;
extern crate hex;
extern crate hkdf;
extern crate sha2;

#[macro_use]
extern crate serde;
extern crate serde_json;

use sgx_types::*;

mod crypto;
mod error;
mod submit;
mod tests;

#[no_mangle]
pub extern "C" fn test_main_entrance() -> sgx_status_t {
    tests::test_all()
}
