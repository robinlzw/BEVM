// Copyright 2023 BEVM Project Authors. Licensed under GPL-3.0.

pub use sc_executor::NativeElseWasmExecutor;

pub struct BevmExecutor;
impl sc_executor::NativeExecutionDispatch for BevmExecutor {
    type ExtendHostFunctions = (
        frame_benchmarking::benchmarking::HostFunctions,
        xp_io::ss_58_codec::HostFunctions,
    );

    fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
        bevm_runtime::api::dispatch(method, data)
    }

    fn native_version() -> sc_executor::NativeVersion {
        bevm_runtime::native_version()
    }
}