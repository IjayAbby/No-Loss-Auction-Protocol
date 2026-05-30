const statusEl = document.getElementById('status-output');
const connectButton = document.getElementById('connect-button');
const createButton = document.getElementById('create-auction-button');
const placeBidButton = document.getElementById('place-bid-button');
const finalizeButton = document.getElementById('finalize-auction-button');
const cancelButton = document.getElementById('cancel-auction-button');
const claimRefundButton = document.getElementById('claim-refund-button');
const loadAuctionButton = document.getElementById('load-auction-button');

let walletKeypair = null;
let contractId = null;
let tokenId = null;
let server = null;
let networkPassphrase = null;

function status(message) {
  statusEl.textContent = `${new Date().toISOString()} - ${message}`;
}

function connectWallet() {
  const secret = document.getElementById('wallet-secret').value.trim();
  contractId = document.getElementById('contract-id').value.trim();
  tokenId = document.getElementById('token-id').value.trim();
  if (!secret || !contractId || !tokenId) {
    status('Enter secret, contract ID, and token contract ID first.');
    return;
  }
  walletKeypair = StellarSdk.Keypair.fromSecret(secret);
  server = new SorobanClient.Server('https://soroban-testnet.stellar.org');
  networkPassphrase = SorobanClient.Networks.TESTNET;
  status('Wallet connected: ' + walletKeypair.publicKey());
}

async function buildAccount() {
  const account = await server.loadAccount(walletKeypair.publicKey());
  return account;
}

function encodeBytes32(hexString) {
  const bytes = hexString.match(/.{1,2}/g).map(byte => parseInt(byte, 16));
  return SorobanClient.xdr.ScVal.scvBytes(bytes);
}

async function sendContractInvocation(functionName, args) {
  const account = await buildAccount();
  const tx = new StellarSdk.TransactionBuilder(account, {
    fee: '100',
    networkPassphrase,
  })
    .addOperation(SorobanClient.Operation.invokeHostFunction({
      function: SorobanClient.HostFunction.hostFunctionTypeInvokeContract(),
      contractId,
      functionName,
      args,
    }))
    .setTimeout(60)
    .build();
  tx.sign(walletKeypair);
  const response = await server.sendTransaction(tx);
  return response;
}

connectButton.onclick = connectWallet;
createButton.onclick = async () => {
  try {
    const auctionId = document.getElementById('create-auction-id').value.trim();
    const deadline = Number(document.getElementById('create-deadline').value);
    const minimumBid = Number(document.getElementById('create-minimum-bid').value);
    if (!auctionId || !deadline || !minimumBid) {
      status('Please fill create auction fields.');
      return;
    }
    const args = [
      encodeBytes32(auctionId),
      SorobanClient.xdr.ScVal.scvAddress(SorobanClient.xdr.PublicKey.publicKeyTypeEd25519(walletKeypair.rawPublicKey())),
      SorobanClient.xdr.ScVal.scvObject(SorobanClient.xdr.BytesN.fromXDR(Buffer.from(tokenId, 'hex'))),
      SorobanClient.xdr.ScVal.scvI64(deadline),
      SorobanClient.xdr.ScVal.scvI128(minimumBid),
    ];
    status('Sending create auction transaction...');
    const response = await sendContractInvocation('create_auction', args);
    status('Auction created: ' + JSON.stringify(response));
  } catch (error) {
    status('Create auction error: ' + error.toString());
  }
};
placeBidButton.onclick = async () => {
  try {
    const auctionId = document.getElementById('bid-auction-id').value.trim();
    const amount = Number(document.getElementById('bid-amount').value);
    if (!auctionId || !amount) {
      status('Please fill bid fields.');
      return;
    }
    const args = [encodeBytes32(auctionId), SorobanClient.xdr.ScVal.scvI128(amount)];
    status('Sending bid transaction...');
    const response = await sendContractInvocation('place_bid', args);
    status('Bid placed: ' + JSON.stringify(response));
  } catch (error) {
    status('Bid error: ' + error.toString());
  }
};
finalizeButton.onclick = async () => {
  try {
    const auctionId = document.getElementById('finalize-auction-id').value.trim();
    if (!auctionId) {
      status('Enter auction ID to finalize.');
      return;
    }
    const args = [encodeBytes32(auctionId)];
    status('Finalizing auction...');
    const response = await sendContractInvocation('finalize_auction', args);
    status('Auction finalized: ' + JSON.stringify(response));
  } catch (error) {
    status('Finalize error: ' + error.toString());
  }
};
cancelButton.onclick = async () => {
  try {
    const auctionId = document.getElementById('finalize-auction-id').value.trim();
    if (!auctionId) {
      status('Enter auction ID to cancel.');
      return;
    }
    const args = [encodeBytes32(auctionId)];
    status('Cancelling auction...');
    const response = await sendContractInvocation('cancel_auction', args);
    status('Auction cancelled: ' + JSON.stringify(response));
  } catch (error) {
    status('Cancel error: ' + error.toString());
  }
};
claimRefundButton.onclick = async () => {
  try {
    const auctionId = document.getElementById('refund-auction-id').value.trim();
    if (!auctionId) {
      status('Enter auction ID to claim refund.');
      return;
    }
    const args = [encodeBytes32(auctionId)];
    status('Claiming refund...');
    const response = await sendContractInvocation('claim_refund', args);
    status('Refund claimed: ' + JSON.stringify(response));
  } catch (error) {
    status('Refund error: ' + error.toString());
  }
};
loadAuctionButton.onclick = async () => {
  status('Load auction not implemented in UI. Use contract explorer or read-only call support in your Soroban client.');
};
