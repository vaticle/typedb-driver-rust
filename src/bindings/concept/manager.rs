/*
 * Copyright (C) 2022 Vaticle
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

use std::ffi::c_char;

use crate::{
    bindings::{
        error::{try_release, try_release_map_optional},
        memory::{borrow, string_view},
    },
    common::IID,
    concept::{Concept, ValueType},
    Transaction,
};

#[no_mangle]
pub extern "C" fn concepts_get_entity_type(
    transaction: *const Transaction<'static>,
    label: *const c_char,
) -> *mut Concept {
    try_release_map_optional(
        borrow(transaction).concept().get_entity_type(string_view(label).to_owned()).transpose(),
        Concept::EntityType,
    )
}

#[no_mangle]
pub extern "C" fn concepts_get_relation_type(
    transaction: *const Transaction<'static>,
    label: *const c_char,
) -> *mut Concept {
    try_release_map_optional(
        borrow(transaction).concept().get_relation_type(string_view(label).to_owned()).transpose(),
        Concept::RelationType,
    )
}

#[no_mangle]
pub extern "C" fn concepts_get_attribute_type(
    transaction: *const Transaction<'static>,
    label: *const c_char,
) -> *mut Concept {
    try_release_map_optional(
        borrow(transaction).concept().get_attribute_type(string_view(label).to_owned()).transpose(),
        Concept::AttributeType,
    )
}

#[no_mangle]
pub extern "C" fn concepts_put_entity_type(
    transaction: *const Transaction<'static>,
    label: *const c_char,
) -> *mut Concept {
    try_release(borrow(transaction).concept().put_entity_type(string_view(label).to_owned()).map(Concept::EntityType))
}

#[no_mangle]
pub extern "C" fn concepts_put_relation_type(
    transaction: *const Transaction<'static>,
    label: *const c_char,
) -> *mut Concept {
    try_release(
        borrow(transaction).concept().put_relation_type(string_view(label).to_owned()).map(Concept::RelationType),
    )
}

#[no_mangle]
pub extern "C" fn concepts_put_attribute_type(
    transaction: *const Transaction<'static>,
    label: *const c_char,
    value_type: ValueType,
) -> *mut Concept {
    try_release(
        borrow(transaction)
            .concept()
            .put_attribute_type(string_view(label).to_owned(), value_type)
            .map(Concept::AttributeType),
    )
}

fn iid_from_str(str: &str) -> IID {
    (2..str.len()).step_by(2).map(|i| u8::from_str_radix(&str[i..i + 2], 16).unwrap()).collect::<Vec<u8>>().into()
}

#[no_mangle]
pub extern "C" fn concepts_get_entity(transaction: *const Transaction<'static>, iid: *const c_char) -> *mut Concept {
    try_release_map_optional(
        borrow(transaction).concept().get_entity(iid_from_str(string_view(iid))).transpose(),
        Concept::Entity,
    )
}

#[no_mangle]
pub extern "C" fn concepts_get_relation(transaction: *const Transaction<'static>, iid: *const c_char) -> *mut Concept {
    try_release_map_optional(
        borrow(transaction).concept().get_relation(iid_from_str(string_view(iid))).transpose(),
        Concept::Relation,
    )
}

#[no_mangle]
pub extern "C" fn concepts_get_attribute(transaction: *const Transaction<'static>, iid: *const c_char) -> *mut Concept {
    try_release_map_optional(
        borrow(transaction).concept().get_attribute(iid_from_str(string_view(iid))).transpose(),
        Concept::Attribute,
    )
}

#[no_mangle]
pub extern "C" fn concepts_get_schema_exceptions(transaction: *const Transaction<'static>) {
    borrow(transaction).concept().get_schema_exceptions().ok();
}