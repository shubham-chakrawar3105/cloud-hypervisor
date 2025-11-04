// Copyright © 2019 Intel Corporation
//
// SPDX-License-Identifier: Apache-2.0 OR BSD-3-Clause
//
// Copyright © 2020, Microsoft Corporation
//
// Copyright 2018-2019 CrowdStrike, Inc.
//
//
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64;
use std::sync::Arc;

use thiserror::Error;

#[cfg(target_arch = "x86_64")]
use crate::arch::x86::CpuIdEntry;
#[cfg(target_arch = "x86_64")]
use crate::cpu::CpuVendor;
#[cfg(feature = "tdx")]
use crate::kvm::TdxCapabilities;
use crate::vm::Vm;
use crate::{HypervisorType, HypervisorVmConfig};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to check availability of the hypervisor: {0:?}")]
    HypervisorAvailableCheck(#[source] std::io::Error),
    #[error("Failed to create the hypervisor: {0:?}")]
    HypervisorCreate(#[source] std::io::Error),
    #[error("Failed to get API Version: {0:?}")]
    GetApiVersion(#[source] std::io::Error),
    #[error("Checking extensions: {0:?}")]
    CheckExtensions(#[source] std::io::Error),
    #[error("Failed to get cpuid: {0:?}")]
    GetCpuId(#[source] std::io::Error),
    #[error("Failed to get the list of supported MSRs: {0:?}")]
    GetMsrList(#[source] std::io::Error),
    #[error("Failed to retrieve TDX capabilities: {0:?}")]
    TdxCapabilities(#[source] std::io::Error),
    #[error("Failed to set partition property: {0:?}")]
    SetPartitionProperty(#[source] std::io::Error),
    #[error("Unsupported CPU")]
    UnsupportedCpu,
}

#[derive(Error, Debug)]
pub enum VmError {
    #[error("Failed to create Vm: {0:?}")]
    VmCreate(#[source] std::io::Error),
    #[error("Failed to setup Vm: {0:?}")]
    VmSetup(#[source] std::io::Error),
}

#[derive(Error, Debug)]
pub enum HypervisorError {
    ///
    /// Hypervisor availability check error
    ///
    #[error("Failed to check availability of the hypervisor")]
    HypervisorAvailableCheck(#[source] Error),
    ///
    /// hypervisor creation error
    ///
    #[error("Failed to create the hypervisor")]
    HypervisorCreate(#[source] Error),
    ///
    /// Vm creation failure
    ///
    #[error("Failed to create Vm")]
    VmCreate(#[source] VmError),
    ///
    /// Vm setup failure
    ///
    #[error("Failed to setup Vm")]
    VmSetup(#[source] VmError),
    ///
    /// API version error
    ///
    #[error("Failed to get API Version")]
    GetApiVersion(#[source] Error),
    ///
    /// CpuId error
    ///
    #[error("Failed to get cpuid")]
    GetCpuId(#[source] Error),
    ///
    /// Failed to retrieve list of MSRs.
    ///
    #[error("Failed to get the list of supported MSRs")]
    GetMsrList(#[source] Error),
    ///
    /// API version is not compatible
    ///
    #[error("Incompatible API version")]
    IncompatibleApiVersion,
    ///
    /// Checking extensions failed
    ///
    #[error("Checking extensions")]
    CheckExtensions(#[source] Error),
    ///
    /// Failed to retrieve TDX capabilities
    ///
    #[error("Failed to retrieve TDX capabilities")]
    TdxCapabilities(#[source] Error),
    ///
    /// Failed to set partition property
    ///
    #[error("Failed to set partition property")]
    SetPartitionProperty(#[source] Error),
    ///
    /// Running on an unsupported CPU
    ///
    #[error("Unsupported CPU")]
    UnsupportedCpu(#[source] Error),
    ///
    /// Launching a VM with unsupported VM Type
    ///
    #[error("Unsupported VmType")]
    UnsupportedVmType(),
}

///
/// Result type for returning from a function
///
pub type Result<T> = std::result::Result<T, HypervisorError>;

///
/// Trait to represent a Hypervisor
///
/// This crate provides a hypervisor-agnostic interfaces
///
pub trait Hypervisor: Send + Sync {
    ///
    /// Returns the type of the hypervisor
    ///
    fn hypervisor_type(&self) -> HypervisorType;
    ///
    /// Create a Vm using the underlying hypervisor
    /// Return a hypervisor-agnostic Vm trait object
    ///
    fn create_vm(&self, config: HypervisorVmConfig) -> Result<Arc<dyn Vm>>;
    #[cfg(target_arch = "x86_64")]
    ///
    /// Get the supported CpuID
    ///
    fn get_supported_cpuid(&self) -> Result<Vec<CpuIdEntry>>;
    ///
    /// Check particular extensions if any
    ///
    fn check_required_extensions(&self) -> Result<()> {
        Ok(())
    }
    #[cfg(target_arch = "aarch64")]
    ///
    /// Retrieve AArch64 host maximum IPA size supported by KVM
    ///
    fn get_host_ipa_limit(&self) -> i32;
    ///
    /// Retrieve TDX capabilities
    ///
    #[cfg(feature = "tdx")]
    fn tdx_capabilities(&self) -> Result<TdxCapabilities> {
        unimplemented!()
    }
    ///
    /// Get the number of supported hardware breakpoints
    ///
    fn get_guest_debug_hw_bps(&self) -> usize {
        unimplemented!()
    }

    /// Get maximum number of vCPUs
    fn get_max_vcpus(&self) -> u32;
    #[cfg(target_arch = "x86_64")]
    ///
    /// Determine CPU vendor
    ///
    fn get_cpu_vendor(&self) -> CpuVendor {
        // SAFETY: call cpuid with valid leaves
        unsafe {
            let leaf = x86_64::__cpuid(0x0);

            if leaf.ebx == 0x756e_6547 && leaf.ecx == 0x6c65_746e && leaf.edx == 0x4965_6e69 {
                // Vendor string GenuineIntel
                CpuVendor::Intel
            } else if leaf.ebx == 0x6874_7541 && leaf.ecx == 0x444d_4163 && leaf.edx == 0x6974_6e65
            {
                // Vendor string AuthenticAMD
                CpuVendor::AMD
            } else {
                // Not known yet, the corresponding manufacturer manual should contain the
                // necessary info. See also https://wiki.osdev.org/CPUID#CPU_Vendor_ID_String
                CpuVendor::default()
            }
        }
    }
}
