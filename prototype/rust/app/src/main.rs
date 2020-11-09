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

extern crate sgx_types;
extern crate sgx_urts;

use sgx_types::*;
use sgx_urts::SgxEnclave;

extern crate interface;
extern crate serde_json;

mod enclave_tests;
mod enclave_wrapper;
mod utils;

use enclave_wrapper::*;
use interface::*;

use sgx_status_t::SGX_SUCCESS;

fn main() {
    let dc_enclave = match DcNetEnclave::init() {
        Ok(r) => {
            println!("[+] Init Enclave Successful {}!", r.geteid());
            r
        }
        Err(x) => {
            println!("[-] Init Enclave Failed {}!", x.as_str());
            return;
        }
    };

    // enclave_tests::test(&enclave);

    let send_request = SendRequest {
        message: [9 as u8; DC_NET_MESSAGE_LENGTH],
        round: 0,
        server_keys: vec![ServerSecret::gen_test(1), ServerSecret::gen_test(2)],
    };

    let sgx_key = PrvKey::gen_test(9);

    match dc_enclave.client_submit(&send_request, &sgx_key) {
        Ok(m) => println!("{:?}", m),
        Err(e) => println!("Err {}", e),
    }

    dc_enclave.close();
}
