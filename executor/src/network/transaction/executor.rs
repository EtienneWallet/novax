use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;

use async_trait::async_trait;
use multiversx_sc::codec::{TopDecodeMulti, TopEncodeMulti};
use multiversx_sc_scenario::scenario_model::TypedScDeploy;
use num_bigint::BigUint;

use novax_data::{Address, NativeConvertible};

use crate::base::deploy::DeployExecutor;
use crate::base::transaction::TransactionExecutor;
use crate::call_result::CallResult;
use crate::error::executor::ExecutorError;
use crate::error::transaction::TransactionError;
use crate::network::transaction::interactor::{BlockchainInteractor, Interactor};
use crate::network::utils::wallet::Wallet;
use crate::{TransactionOnNetwork, TransactionOnNetworkTransactionSmartContractResult};
use crate::utils::transaction::normalization::NormalizationInOut;
use crate::utils::transaction::token_transfer::TokenTransfer;

/// Alias for the `BaseTransactionNetworkExecutor` struct, parameterized with the `Interactor` type.
pub type NetworkExecutor = BaseTransactionNetworkExecutor<Interactor>;

/// A struct representing the executor for handling transactions in a real blockchain environment.
///
/// This executor is designed to interact with a blockchain network via a specified gateway URL and a wallet
/// for signing transactions. It is parameterized by a type `Interactor` that encapsulates the blockchain interaction logic.
pub struct BaseTransactionNetworkExecutor<Interactor: BlockchainInteractor> {
    /// The URL of the blockchain network gateway through which transactions will be sent.
    pub gateway_url: String,
    /// The wallet used for signing transactions before they are sent to the blockchain network.
    pub wallet: Wallet,
    /// Phantom data to allow the generic parameter `Interactor`.
    /// This field does not occupy any space in memory.
    _phantom_data: PhantomData<Interactor>,
}

/// Custom implementation of `Clone` for `BaseTransactionNetworkExecutor`.
///
/// This implementation is necessary because the `Interactor` generic parameter might not
/// implement `Clone`. However, since `Interactor` is used only as phantom data (it does not
/// affect the state of `BaseTransactionNetworkExecutor`), we can safely implement `Clone`
/// without the `Interactor` needing to be `Clone`.
impl<Interactor> Clone for BaseTransactionNetworkExecutor<Interactor>
    where
        Interactor: BlockchainInteractor
{
    fn clone(&self) -> Self {
        Self {
            gateway_url: self.gateway_url.clone(),
            wallet: self.wallet,
            _phantom_data: Default::default(),
        }
    }
}

/// Custom implementation of `Debug` for `BaseTransactionNetworkExecutor`.
///
/// This implementation is necessary because the `Interactor` generic parameter might not
/// implement `Debug`. As with `Clone`, since `Interactor` is only used as phantom data,
/// it does not impact the debug representation of `BaseTransactionNetworkExecutor`. This
/// implementation ensures that instances of `BaseTransactionNetworkExecutor` can be
/// formatted using the `Debug` trait regardless of whether `Interactor` implements `Debug`.
impl<Interactor> Debug for BaseTransactionNetworkExecutor<Interactor>
    where
        Interactor: BlockchainInteractor
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BaseTransactionNetworkExecutor")
            .field("gateway_url", &self.gateway_url)
            .field("wallet", &self.wallet)
            .finish()
    }
}

impl<Interactor: BlockchainInteractor> BaseTransactionNetworkExecutor<Interactor> {
    /// Creates a new instance of `BaseTransactionNetworkExecutor`.
    ///
    /// # Parameters
    /// - `gateway_url`: The URL of the blockchain network gateway.
    /// - `wallet`: A reference to the wallet used for signing transactions.
    ///
    /// # Returns
    /// A new `BaseTransactionNetworkExecutor` instance.
    pub fn new(gateway_url: &str, wallet: &Wallet) -> Self {
        BaseTransactionNetworkExecutor {
            gateway_url: gateway_url.to_string(),
            wallet: *wallet,
            _phantom_data: PhantomData,
        }
    }
}

#[async_trait]
impl<Interactor: BlockchainInteractor> TransactionExecutor for BaseTransactionNetworkExecutor<Interactor> {
    async fn sc_call<OutputManaged>(
        &mut self,
        to: &Address,
        function: String,
        arguments: Vec<Vec<u8>>,
        gas_limit: u64,
        egld_value: BigUint,
        esdt_transfers: Vec<TokenTransfer>
    ) -> Result<CallResult<OutputManaged::Native>, ExecutorError>
        where
            OutputManaged: TopDecodeMulti + NativeConvertible + Send + Sync
    {
        let mut interactor = Interactor::new(
            self.gateway_url.clone(),
            self.wallet
        )
            .await?;

        let normalized = NormalizationInOut {
            sender: self.wallet.get_address().to_bech32_string()?,
            receiver: to.to_bech32_string()?,
            function_name: function,
            arguments,
            egld_value,
            esdt_transfers,
        }.normalize()?;

        let receiver = normalized.receiver.clone();
        let egld_value = normalized.egld_value.clone();
        let transaction_data = normalized.get_transaction_data();

        let result = interactor.sc_call(
            receiver,
            egld_value,
            transaction_data,
            gas_limit,
        )
            .await?;

        let Some(mut sc_result) = find_smart_contract_result(&result.transaction.smart_contract_results) else {
            return Err(TransactionError::NoSmartContractResult.into())
        };

        let managed_result = OutputManaged::multi_decode(&mut sc_result)
            .map_err(|_| TransactionError::CannotDecodeSmartContractResult)?;

        let native_result = managed_result.to_native();

        let call_result = CallResult {
            response: result,
            result: Some(native_result),
        };

        Ok(call_result)
    }
}

/// Implementation of the `DeployExecutor` trait for the `BaseTransactionNetworkExecutor` struct.
/// This implementation enables the deployment of smart contracts on the blockchain
/// using a specified blockchain interactor.
#[async_trait]
impl<Interactor: BlockchainInteractor> DeployExecutor for BaseTransactionNetworkExecutor<Interactor> {

    /// Asynchronously deploys a smart contract to the blockchain.
    ///
    /// # Type Parameters
    ///
    /// * `OriginalResult`: Represents the result type expected from the smart contract deployment.
    ///    This type must implement `TopEncodeMulti`, `Send`, and `Sync`.
    /// * `S`: Represents the type encapsulating the smart contract deployment step.
    ///    This type must implement `AsMut<TypedScDeploy<OriginalResult>>` and `Send`.
    ///
    /// # Parameters
    ///
    /// * `sc_deploy_step`: A mutable reference to the smart contract deployment step to be executed.
    ///
    /// # Returns
    ///
    /// A `Result` with an empty `Ok(())` value indicating success, or an `Err(ExecutorError)` indicating failure.
    async fn sc_deploy<OriginalResult>(&mut self, sc_deploy_step: &mut TypedScDeploy<OriginalResult>) -> Result<(), ExecutorError>
        where
            OriginalResult: TopEncodeMulti + Send + Sync,
    {
        todo!()
    }

    /// Specifies whether deserialization should be skipped during the deployment execution.
    /// In this implementation, deserialization is not skipped.
    ///
    /// # Returns
    ///
    /// A `bool` value of `false`, indicating that deserialization should not be skipped.
    async fn should_skip_deserialization(&self) -> bool {
        false
    }
}

fn find_smart_contract_result(opt_sc_results: &Option<Vec<TransactionOnNetworkTransactionSmartContractResult>>) -> Option<Vec<Vec<u8>>> {
    let Some(sc_results) = opt_sc_results else {
        return None
    };

    sc_results.iter()
        .find(|sc_result| sc_result.nonce != 0 && sc_result.data.starts_with('@'))
        .cloned()
        .map(|sc_result| {
            let mut split = sc_result.data.split('@');
            let _ = split.next().expect("SCR data should start with '@'"); // TODO: no expect and assert_eq!
            let result_code = split.next().expect("missing result code");
            assert_eq!(result_code, "6f6b", "result code is not 'ok'");

            split
                .map(|encoded_arg| hex::decode(encoded_arg).expect("error hex-decoding result"))
                .collect()
        })
}