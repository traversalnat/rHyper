use aarch64_cpu::registers::{ESR_EL2, FAR_EL2};
use rvm::{RvmResult, RvmVcpu};
use tock_registers::interfaces::Readable;

use crate::{hv::device_emu::all_virt_devices, device::pending_irq};

use super::hal::RvmHalImpl;

type Vcpu = RvmVcpu<RvmHalImpl>;

fn handle_hypercall(vcpu: &mut Vcpu) -> RvmResult {
    let regs = vcpu.regs();
    info!(
        "VM exit: VMCALL({:#x}): {:?}",
        regs.x[0],
        [regs.x[1], regs.x[2], regs.x[3], regs.x[4]]
    );
    match regs.x[0] {
        PSCI_CPU_OFF  => {
            loop {}
        },
        _ => {}
    }
    Ok(())
}

fn handle_iabt(vcpu: &mut Vcpu) -> RvmResult {
    // todo!();
    let regs = vcpu.regs();
    // info!("VTTBR_EL2: {:x}", VTTBR_EL2.get());
    // vcpu.advance_rip()?;
    // Ok(())
    Err(rvm::RvmError::ResourceBusy)
}

fn handle_dabt(vcpu: &mut Vcpu) -> RvmResult {
    // we need to add HPFAR_EL2 to aarch64_cpu
    // FAR_EL2 val is not correct, we use it temporarily
    let fault_vaddr = FAR_EL2.get();
    if let Some(dev) = all_virt_devices().find_mmio_device(fault_vaddr as usize) {
        // decode the instruction by hand....
        
        Ok(())
    } else {
        Err(rvm::RvmError::OutOfMemory)
    }
}

#[no_mangle]
pub fn vmexit_handler(vcpu: &mut Vcpu) -> RvmResult {
    let exit_info = vcpu.exit_info()?;
    // debug!("VM exit: {:#x?}", exit_info);

    let res = match exit_info.exit_reason {
        Some(ESR_EL2::EC::Value::HVC64) => handle_hypercall(vcpu),
        Some(ESR_EL2::EC::Value::InstrAbortLowerEL) => handle_iabt(vcpu),
        Some(ESR_EL2::EC::Value::InstrAbortCurrentEL) => handle_iabt(vcpu),
        Some(ESR_EL2::EC::Value::DataAbortLowerEL) => handle_dabt(vcpu),
        Some(ESR_EL2::EC::Value::DataAbortCurrentEL) => handle_dabt(vcpu),
        _ => panic!(
            "Unhandled VM-Exit reason {:?}:\n{:#x?}",
            exit_info.exit_reason.unwrap() as u64,
            vcpu
        ),
    };

    if res.is_err() {
        panic!(
            "Failed to handle VM-exit {:?}:\n{:#x?}",
            exit_info.exit_reason.unwrap() as u64,
            vcpu
        );
    }

    Ok(())
}

#[no_mangle]
pub fn irq_handler() -> RvmResult {
    // info!("IRQ routed to EL2");
    if let Some(irq_id) = pending_irq() {
        info!("IRQ {} routed to EL2", irq_id);
    }
    Ok(())
    // // let irq_number =
    // todo!()
}
