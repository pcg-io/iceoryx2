// Copyright (c) 2024 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache Software License 2.0 which is available at
// https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
// which is available at https://opensource.org/licenses/MIT.
//
// SPDX-License-Identifier: Apache-2.0 OR MIT

#![allow(non_camel_case_types)]

use iceoryx2::service::header::publish_subscribe::Header;
use iceoryx2_bb_elementary::static_assert::static_assert_ge;
use iceoryx2_ffi_macros::iceoryx2_ffi;

use crate::{iox2_unique_publisher_id_h, iox2_unique_publisher_id_t};

use super::HandleToType;

// BEGIN types definition

/// Sample header used by `MessagingPattern::PublishSubscribe`
#[repr(C)]
#[repr(align(8))] // core::mem::align_of::<Option<Header>>()
pub struct iox2_publish_subscribe_header_storage_t {
    internal: [u8; 32], // core::mem::size_of::<Option<Header>>()
}

#[repr(C)]
#[iceoryx2_ffi(Header)]
pub struct iox2_publish_subscribe_header_t {
    pub value: iox2_publish_subscribe_header_storage_t,
    deleter: fn(*mut iox2_publish_subscribe_header_t),
}

impl iox2_publish_subscribe_header_t {
    pub(super) fn init(
        &mut self,
        header: Header,
        deleter: fn(*mut iox2_publish_subscribe_header_t),
    ) {
        self.value.init(header);
        self.deleter = deleter;
    }
}

pub struct iox2_publish_subscribe_header_h_t;
/// The owning handle for [`iox2_publish_subscribe_header_t`]. Passing the handle to an function transfers the ownership.
pub type iox2_publish_subscribe_header_h = *mut iox2_publish_subscribe_header_h_t;

pub struct iox2_publish_subscribe_header_ref_h_t;
/// The non-owning handle for [`iox2_publish_subscribe_header_t`]. Passing the handle to an function does not transfers the ownership.
pub type iox2_publish_subscribe_header_ref_h = *mut iox2_publish_subscribe_header_ref_h_t;

// NOTE check the README.md for using opaque types with renaming
/// The immutable pointer to the underlying `publish_subscribe::Header`
pub type iox2_publish_subscribe_header_ptr = *const Header;
/// The mutable pointer to the underlying `publish_subscribe::Header`
pub type iox2_publish_subscribe_header_mut_ptr = *mut Header;

impl HandleToType for iox2_publish_subscribe_header_h {
    type Target = *mut iox2_publish_subscribe_header_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

impl HandleToType for iox2_publish_subscribe_header_ref_h {
    type Target = *mut iox2_publish_subscribe_header_t;

    fn as_type(self) -> Self::Target {
        self as *mut _ as _
    }
}

// END types definition

// BEGIN C API

/// This function casts an owning [`iox2_publish_subscribe_header_h`] into a non-owning [`iox2_publish_subscribe_header_ref_h`]
///
/// Returns a [`iox2_publish_subscribe_header_ref_h`]
///
/// # Safety
///
/// * The `handle` must be a valid handle.
/// * The `handle` is still valid after the call to this function.
#[no_mangle]
pub unsafe extern "C" fn iox2_cast_publish_subscribe_header_ref_h(
    handle: iox2_publish_subscribe_header_h,
) -> iox2_publish_subscribe_header_ref_h {
    debug_assert!(!handle.is_null());

    (*handle.as_type()).as_ref_handle() as *mut _ as _
}

/// This function needs to be called to destroy the publish_subscribe_header!
///
/// # Safety
///
/// * The `handle` is invalid after the return of this function and leads to undefined behavior if used in another function call!
/// * The corresponding [`iox2_publish_subscribe_header_t`] can be re-used
#[no_mangle]
pub unsafe extern "C" fn iox2_publish_subscribe_header_drop(
    handle: iox2_publish_subscribe_header_h,
) {
    debug_assert!(!handle.is_null());

    let header = &mut *handle.as_type();
    core::ptr::drop_in_place(header.value.as_option_mut());

    (header.deleter)(header);
}

/// Returns the unique publisher id of the source of the sample.
///
/// # Arguments
///
/// * `handle` is valid, non-null and was initialized with
///    [`iox2_sample_header()`](crate::iox2_sample_header)
/// * `id_struct_ptr` - Must be either a NULL pointer or a pointer to a valid [`iox2_unique_publisher_id_t`].
///                         If it is a NULL pointer, the storage will be allocated on the heap.
/// * `id_handle_ptr` valid pointer to a [`iox2_unique_publisher_id_h`].
///
/// # Safety
///
/// * `header_handle` is valid, non-null and was obtained via [`iox2_cast_publish_subscribe_header_ref_h`]
/// * `id_struct_ptr` is either null or valid and non-null
/// * `id_handle_ptr` is valid and non-null
#[no_mangle]
pub unsafe extern "C" fn iox2_publish_subscribe_header_publisher_id(
    header_handle: iox2_publish_subscribe_header_ref_h,
    id_struct_ptr: *mut iox2_unique_publisher_id_t,
    id_handle_ptr: *mut iox2_unique_publisher_id_h,
) {
    debug_assert!(!header_handle.is_null());
    debug_assert!(!id_handle_ptr.is_null());

    fn no_op(_: *mut iox2_unique_publisher_id_t) {}
    let mut deleter: fn(*mut iox2_unique_publisher_id_t) = no_op;
    let mut storage_ptr = id_struct_ptr;
    if id_struct_ptr.is_null() {
        deleter = iox2_unique_publisher_id_t::dealloc;
        storage_ptr = iox2_unique_publisher_id_t::alloc();
    }
    debug_assert!(!storage_ptr.is_null());

    let header = &mut *header_handle.as_type();

    let id = header.value.as_ref().publisher_id();

    (*storage_ptr).init(id, deleter);
    *id_handle_ptr = (*storage_ptr).as_handle();
}

/// Returns the payloads type size.
///
/// # Arguments
///
/// * `handle` is valid, non-null and was initialized with
///    [`iox2_sample_header()`](crate::iox2_sample_header)
///
/// # Safety
///
/// * `header_handle` is valid, non-null and was obtained via [`iox2_cast_publish_subscribe_header_ref_h`]
#[no_mangle]
pub unsafe extern "C" fn iox2_publish_subscribe_header_payload_type_size(
    header_handle: iox2_publish_subscribe_header_ref_h,
) -> usize {
    debug_assert!(!header_handle.is_null());

    let header = &mut *header_handle.as_type();

    header.value.as_ref().payload_type_layout().size()
}

/// Returns the payloads type alignment.
///
/// # Arguments
///
/// * `handle` is valid, non-null and was initialized with
///    [`iox2_sample_header()`](crate::iox2_sample_header)
///
/// # Safety
///
/// * `header_handle` is valid, non-null and was obtained via [`iox2_cast_publish_subscribe_header_ref_h`]
#[no_mangle]
pub unsafe extern "C" fn iox2_publish_subscribe_header_payload_type_alignment(
    header_handle: iox2_publish_subscribe_header_ref_h,
) -> usize {
    debug_assert!(!header_handle.is_null());

    let header = &mut *header_handle.as_type();

    header.value.as_ref().payload_type_layout().align()
}

// END C API
