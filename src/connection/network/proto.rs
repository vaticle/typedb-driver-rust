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

use std::collections::HashMap;

use chrono::NaiveDateTime;
use typedb_protocol::{
    attribute as attribute_proto,
    attribute_type::ValueType,
    concept as concept_proto,
    numeric::Value,
    options::{
        ExplainOpt::Explain, InferOpt::Infer, ParallelOpt::Parallel, PrefetchOpt::Prefetch,
        PrefetchSizeOpt::PrefetchSize, ReadAnyReplicaOpt::ReadAnyReplica,
        SchemaLockAcquireTimeoutOpt::SchemaLockAcquireTimeoutMillis,
        SessionIdleTimeoutOpt::SessionIdleTimeoutMillis, TraceInferenceOpt::TraceInference,
        TransactionTimeoutOpt::TransactionTimeoutMillis,
    },
    r#type::Encoding,
    session, transaction, Concept as ConceptProto, ConceptMap as ConceptMapProto,
    Numeric as NumericProto, Options as OptionsProto, Thing as ThingProto, Type as TypeProto,
};

use crate::{
    answer::{ConceptMap, Numeric},
    concept::{
        Attribute, AttributeType, BooleanAttribute, BooleanAttributeType, Concept,
        DateTimeAttribute, DateTimeAttributeType, DoubleAttribute, DoubleAttributeType, Entity,
        EntityType, LongAttribute, LongAttributeType, Relation, RelationType, RoleType,
        RootAttributeType, RootThingType, ScopedLabel, StringAttribute, StringAttributeType, Thing,
        ThingType, Type,
    },
    error::ClientError,
    Options, Result, SessionType, TransactionType,
};

pub(super) trait IntoProto {
    type Proto;
    fn into_proto(self) -> Self::Proto;
}

impl IntoProto for SessionType {
    type Proto = session::Type;
    fn into_proto(self) -> Self::Proto {
        match self {
            SessionType::Data => session::Type::Data,
            SessionType::Schema => session::Type::Schema,
        }
    }
}

impl IntoProto for TransactionType {
    type Proto = transaction::Type;
    fn into_proto(self) -> transaction::Type {
        match self {
            TransactionType::Read => transaction::Type::Read,
            TransactionType::Write => transaction::Type::Write,
        }
    }
}

impl IntoProto for Options {
    type Proto = OptionsProto;
    fn into_proto(self) -> Self::Proto {
        OptionsProto {
            infer_opt: self.infer.map(Infer),
            trace_inference_opt: self.trace_inference.map(TraceInference),
            explain_opt: self.explain.map(Explain),
            parallel_opt: self.parallel.map(Parallel),
            prefetch_size_opt: self.prefetch_size.map(PrefetchSize),
            prefetch_opt: self.prefetch.map(Prefetch),
            session_idle_timeout_opt: self
                .session_idle_timeout
                .map(|val| SessionIdleTimeoutMillis(val.as_millis() as i32)),
            transaction_timeout_opt: self
                .transaction_timeout
                .map(|val| TransactionTimeoutMillis(val.as_millis() as i32)),
            schema_lock_acquire_timeout_opt: self
                .schema_lock_acquire_timeout
                .map(|val| SchemaLockAcquireTimeoutMillis(val.as_millis() as i32)),
            read_any_replica_opt: self.read_any_replica.map(ReadAnyReplica),
        }
    }
}

pub(super) trait TryFromProto: Sized {
    type Proto;
    fn try_from_proto(proto: Self::Proto) -> Result<Self>;
}

impl TryFromProto for Numeric {
    type Proto = NumericProto;
    fn try_from_proto(proto: Self::Proto) -> Result<Self> {
        match proto.value.unwrap() {
            Value::LongValue(long) => Ok(Numeric::Long(long)),
            Value::DoubleValue(double) => Ok(Numeric::Double(double)),
            Value::Nan(_) => Ok(Numeric::NaN),
        }
    }
}

impl TryFromProto for ConceptMap {
    type Proto = ConceptMapProto;
    fn try_from_proto(proto: Self::Proto) -> Result<Self> {
        let mut map = HashMap::with_capacity(proto.map.len());
        for (k, v) in proto.map {
            map.insert(k, Concept::try_from_proto(v)?);
        }
        Ok(Self { map })
    }
}

impl TryFromProto for Concept {
    type Proto = ConceptProto;
    fn try_from_proto(proto: Self::Proto) -> Result<Self> {
        let concept = proto.concept.ok_or(ClientError::MissingResponseField("concept"))?;
        match concept {
            concept_proto::Concept::Thing(thing) => Ok(Self::Thing(Thing::try_from_proto(thing)?)),
            concept_proto::Concept::Type(type_) => Ok(Self::Type(Type::try_from_proto(type_)?)),
        }
    }
}

impl TryFromProto for Type {
    type Proto = TypeProto;
    fn try_from_proto(proto: Self::Proto) -> Result<Self> {
        // TODO: replace unwrap() with ok_or(custom_error) throughout the module
        match Encoding::from_i32(proto.encoding).unwrap() {
            Encoding::ThingType => Ok(Self::Thing(ThingType::Root(RootThingType::default()))),
            Encoding::EntityType => {
                Ok(Self::Thing(ThingType::Entity(EntityType::try_from_proto(proto)?)))
            }
            Encoding::RelationType => {
                Ok(Self::Thing(ThingType::Relation(RelationType::try_from_proto(proto)?)))
            }
            Encoding::AttributeType => {
                Ok(Self::Thing(ThingType::Attribute(AttributeType::try_from_proto(proto)?)))
            }
            Encoding::RoleType => Ok(Self::Role(RoleType::try_from_proto(proto)?)),
        }
    }
}

impl TryFromProto for EntityType {
    type Proto = TypeProto;
    fn try_from_proto(proto: Self::Proto) -> Result<Self> {
        Ok(Self::new(proto.label))
    }
}

impl TryFromProto for RelationType {
    type Proto = TypeProto;
    fn try_from_proto(proto: Self::Proto) -> Result<Self> {
        Ok(Self::new(proto.label))
    }
}

impl TryFromProto for AttributeType {
    type Proto = TypeProto;
    fn try_from_proto(proto: Self::Proto) -> Result<Self> {
        match ValueType::from_i32(proto.value_type).unwrap() {
            ValueType::Object => Ok(Self::Root(RootAttributeType::default())),
            ValueType::Boolean => Ok(Self::Boolean(BooleanAttributeType { label: proto.label })),
            ValueType::Long => Ok(Self::Long(LongAttributeType { label: proto.label })),
            ValueType::Double => Ok(Self::Double(DoubleAttributeType { label: proto.label })),
            ValueType::String => Ok(Self::String(StringAttributeType { label: proto.label })),
            ValueType::Datetime => Ok(Self::DateTime(DateTimeAttributeType { label: proto.label })),
        }
    }
}

impl TryFromProto for RoleType {
    type Proto = TypeProto;
    fn try_from_proto(proto: Self::Proto) -> Result<Self> {
        Ok(Self::new(ScopedLabel::new(proto.scope, proto.label)))
    }
}

impl TryFromProto for Thing {
    type Proto = ThingProto;
    fn try_from_proto(proto: Self::Proto) -> Result<Self> {
        match Encoding::from_i32(proto.r#type.clone().unwrap().encoding).unwrap() {
            Encoding::EntityType => Ok(Self::Entity(Entity::try_from_proto(proto)?)),
            Encoding::RelationType => Ok(Self::Relation(Relation::try_from_proto(proto)?)),
            Encoding::AttributeType => Ok(Self::Attribute(Attribute::try_from_proto(proto)?)),
            _ => {
                todo!()
            }
        }
    }
}

impl TryFromProto for Entity {
    type Proto = ThingProto;
    fn try_from_proto(proto: Self::Proto) -> Result<Self> {
        Ok(Self { type_: EntityType::try_from_proto(proto.r#type.unwrap())?, iid: proto.iid })
    }
}

impl TryFromProto for Relation {
    type Proto = ThingProto;
    fn try_from_proto(proto: Self::Proto) -> Result<Self> {
        Ok(Self { type_: RelationType::try_from_proto(proto.r#type.unwrap())?, iid: proto.iid })
    }
}

impl TryFromProto for Attribute {
    type Proto = ThingProto;
    fn try_from_proto(proto: Self::Proto) -> Result<Self> {
        match ValueType::from_i32(proto.r#type.unwrap().value_type).unwrap() {
            ValueType::Object => {
                todo!()
            }
            ValueType::Boolean => Ok(Self::Boolean(BooleanAttribute {
                value: if let attribute_proto::value::Value::Boolean(value) =
                    proto.value.unwrap().value.unwrap()
                {
                    value
                } else {
                    todo!()
                },
                iid: proto.iid,
            })),
            ValueType::Long => Ok(Self::Long(LongAttribute {
                value: if let attribute_proto::value::Value::Long(value) =
                    proto.value.unwrap().value.unwrap()
                {
                    value
                } else {
                    todo!()
                },
                iid: proto.iid,
            })),
            ValueType::Double => Ok(Self::Double(DoubleAttribute {
                value: if let attribute_proto::value::Value::Double(value) =
                    proto.value.unwrap().value.unwrap()
                {
                    value
                } else {
                    todo!()
                },
                iid: proto.iid,
            })),
            ValueType::String => Ok(Self::String(StringAttribute {
                value: if let attribute_proto::value::Value::String(value) =
                    proto.value.unwrap().value.unwrap()
                {
                    value
                } else {
                    todo!()
                },
                iid: proto.iid,
            })),
            ValueType::Datetime => Ok(Self::DateTime(DateTimeAttribute {
                value: if let attribute_proto::value::Value::DateTime(value) =
                    proto.value.unwrap().value.unwrap()
                {
                    NaiveDateTime::from_timestamp_opt(value / 1000, (value % 1000) as u32).unwrap()
                } else {
                    todo!()
                },
                iid: proto.iid,
            })),
        }
    }
}
