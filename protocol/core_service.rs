/*
 * Copyright (C) 2021 Vaticle
 *
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License.  You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied.  See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */

// This file is generated by rust-protobuf 2.8.2. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy::all)]

#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]
//! Generated file from `core/core_service.proto`

use protobuf::Message as Message_imported_for_functions;
use protobuf::ProtobufEnum as ProtobufEnum_imported_for_functions;

/// Generated files are compatible only with the same version
/// of protobuf runtime.
const _PROTOBUF_VERSION_CHECK: () = ::protobuf::VERSION_2_8_2;

static file_descriptor_proto_data: &'static [u8] = b"\
    \n\x17core/core_service.proto\x12\x0ftypedb.protocol\x1a\x18core/core_da\
    tabase.proto\x1a\x14common/session.proto\x1a\x18common/transaction.proto\
    2\x9a\x07\n\x06TypeDB\x12z\n\x12databases_contains\x121.typedb.protocol.\
    CoreDatabaseManager.Contains.Req\x1a1.typedb.protocol.CoreDatabaseManage\
    r.Contains.Res\x12t\n\x10databases_create\x12/.typedb.protocol.CoreDatab\
    aseManager.Create.Req\x1a/.typedb.protocol.CoreDatabaseManager.Create.Re\
    s\x12k\n\rdatabases_all\x12,.typedb.protocol.CoreDatabaseManager.All.Req\
    \x1a,.typedb.protocol.CoreDatabaseManager.All.Res\x12e\n\x0fdatabase_sch\
    ema\x12(.typedb.protocol.CoreDatabase.Schema.Req\x1a(.typedb.protocol.Co\
    reDatabase.Schema.Res\x12e\n\x0fdatabase_delete\x12(.typedb.protocol.Cor\
    eDatabase.Delete.Req\x1a(.typedb.protocol.CoreDatabase.Delete.Res\x12T\n\
    \x0csession_open\x12!.typedb.protocol.Session.Open.Req\x1a!.typedb.proto\
    col.Session.Open.Res\x12W\n\rsession_close\x12\".typedb.protocol.Session\
    .Close.Req\x1a\".typedb.protocol.Session.Close.Res\x12W\n\rsession_pulse\
    \x12\".typedb.protocol.Session.Pulse.Req\x1a\".typedb.protocol.Session.P\
    ulse.Res\x12[\n\x0btransaction\x12#.typedb.protocol.Transaction.Client\
    \x1a#.typedb.protocol.Transaction.Server(\x010\x01B2\n\x1bcom.vaticle.ty\
    pedb.protocolB\x10CoreServiceProto\x88\x01\x01b\x06proto3\
";

static mut file_descriptor_proto_lazy: ::protobuf::lazy::Lazy<::protobuf::descriptor::FileDescriptorProto> = ::protobuf::lazy::Lazy {
    lock: ::protobuf::lazy::ONCE_INIT,
    ptr: 0 as *const ::protobuf::descriptor::FileDescriptorProto,
};

fn parse_descriptor_proto() -> ::protobuf::descriptor::FileDescriptorProto {
    ::protobuf::parse_from_bytes(file_descriptor_proto_data).unwrap()
}

pub fn file_descriptor_proto() -> &'static ::protobuf::descriptor::FileDescriptorProto {
    unsafe {
        file_descriptor_proto_lazy.get(|| {
            parse_descriptor_proto()
        })
    }
}
