use std::mem;
use async_trait::async_trait;
use multiversx_sc::api::{HandleTypeInfo, VMApi};
use multiversx_sc::codec::{TopDecodeMulti, TopEncodeMulti};
use multiversx_sc::imports::{TxEnv, TxFrom, TxGas, TxPayment, TxTo, TxTypedCall};
use multiversx_sc_scenario::scenario_model::{ScCallStep, ScDeployStep, TypedScCall, TypedScDeploy};
use num_bigint::BigUint;
use novax_data::{Address, NativeConvertible};
use crate::base::deploy::DeployExecutor;
use crate::base::transaction::TransactionExecutor;
use crate::call_result::CallResult;
use crate::error::executor::ExecutorError;
use crate::utils::transaction::data::{SendableTransaction, SendableTransactionConvertible};
use crate::utils::transaction::token_transfer::TokenTransfer;

/// A type alias for `DummyExecutor` handling `ScCallStep`.
pub type DummyTransactionExecutor = DummyExecutor<ScCallStep>;

/// A type alias for `DummyExecutor` handling `ScDeployStep`.
pub type DummyDeployExecutor = DummyExecutor<ScDeployStep>;

/// A structure for capturing transaction details without performing actual blockchain transactions.
/// It is designed for testing scenarios, especially to fetch `SendableTransaction` details from interactions.
pub struct DummyExecutor<Tx: SendableTransactionConvertible> {
    /// Holds the transaction details.
    pub tx: Tx,
    /// Optionally holds the caller address.
    pub caller: Option<Address>
}

impl<Tx: SendableTransactionConvertible> DummyExecutor<Tx> {
    /// Retrieves the transaction details encapsulated into a `SendableTransaction`.
    pub fn get_transaction_details(&self) -> SendableTransaction {
        self.tx.to_sendable_transaction()
    }
}

impl<Tx: SendableTransactionConvertible + Default> DummyExecutor<Tx> {
    /// Constructs a new `DummyExecutor` instance.
    ///
    /// # Arguments
    ///
    /// * `caller` - An optional reference to the caller address.
    pub fn new(caller: &Option<Address>) -> DummyExecutor<Tx> {
        DummyExecutor {
            tx: Tx::default(),
            caller: caller.clone()
        }
    }
}

#[async_trait]
impl TransactionExecutor for DummyExecutor<ScCallStep> {
    /// Captures the smart contract call details.
    async fn sc_call<OutputManaged>(
        &mut self,
        to: &Address,
        function: &str,
        arguments: &[&[u8]],
        gas_limit: u64,
        egld_value: &BigUint,
        esdt_transfers: &[TokenTransfer]
    ) -> Result<CallResult<OutputManaged::Native>, ExecutorError>
        where
            OutputManaged: TopDecodeMulti + NativeConvertible + Send + Sync
    {
        /*
        let mut owned_sc_call_step = mem::replace(sc_call_step, ScCallStep::new().into());

        if let Some(caller) = &self.caller {
            owned_sc_call_step = owned_sc_call_step.from(&multiversx_sc::types::Address::from(caller.to_bytes()));
        }

        self.tx = owned_sc_call_step.sc_call_step;

        Ok(())

         */

        todo!()
    }

    /// Indicates that deserialization should be skipped as there is no actual execution.
    async fn should_skip_deserialization(&self) -> bool {
        true
    }
}

#[async_trait]
impl DeployExecutor for DummyExecutor<ScDeployStep> {
    /// Captures the smart contract deployment details.
    async fn sc_deploy<OriginalResult>(&mut self, sc_deploy_step: &mut TypedScDeploy<OriginalResult>) -> Result<(), ExecutorError>
        where
            OriginalResult: TopEncodeMulti + Send + Sync,
    {
        let mut owned_sc_deploy_step = mem::replace(sc_deploy_step, ScDeployStep::new().into());

        if let Some(caller) = &self.caller {
            owned_sc_deploy_step = owned_sc_deploy_step.from(&multiversx_sc::types::Address::from(caller.to_bytes()));
        }

        self.tx = owned_sc_deploy_step.sc_deploy_step;

        Ok(())
    }

    /// Indicates that deserialization should be skipped as there is no actual execution.
    async fn should_skip_deserialization(&self) -> bool {
        true
    }
}