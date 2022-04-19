const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');
const BN = require('bn.js');

const main = async() => {
  const provider = new WsProvider('ws://18.139.30.168:9944');
  //const provider = new HttpProvider('http://localhost:9933');
  const api = await ApiPromise.create({ provider });
  const PHRASE_TEST_ACCOUNT = 'speak sentence monster because comfort feature puppy area team piece plug field';

  const PHRASE_ROOT = 'fish dash budget stairs hire reason mention forest census copper kid away';
  const keyring = new Keyring({ type: 'sr25519' });

  // TEST ACCOUNT
  const TEST_ACCOUNT = keyring.addFromUri(PHRASE_TEST_ACCOUNT);

  // Add an account, straight mnemonic
  const ROOT = keyring.addFromUri(PHRASE_ROOT);
  // Check account balance
  //const alice = '5CaYWkKW8hVBft5uuiQaKsatAnKPsbHBF2oSmQMZ5qAFZ9Et';
  const { nonce, data: balance } = await api.query.system.account(TEST_ACCOUNT.address);
  console.log(`balance of ${balance.free} and a nonce of ${nonce}`);

  // create campaign
  /// Parameter: 
  /// + who: everyone ->sign this extrinsic
  /// + value: amount of token for this campaign
  const unsub = await api.tx.task
  .createCampaign(10000000000)
  .signAndSend(TEST_ACCOUNT, (result) => {
    console.log(`Current status is ${result.status}`);

  if (result.status.isFinalized) {
      console.log(`Transaction finalized at blockHash ${result.status.asFinalized}`);
      console.log("Create campaign successfully");
      unsub();
    }
  });

  // Right now we have campaign index =0

  // Approve campaign
  // should be root key can call this function (sudo key)
  // Parameter:
  // + who: root key ->sign this extrinsic
  // + campaign_index: u32


  const unsub1 = await api.tx.sudo
  .sudo(
    api.tx.task.approveCampaign(0)
  )
  .signAndSend(ROOT, (result) => {
    if (result.status.isFinalized) {
      console.log(`Transaction finalized at blockHash ${result.status.asFinalized}`);
      console.log("Sudo key approve this campaign");
      unsub1();
    }

  });

  /// Campain should be approved
  // Parameter:
  // + who: root key ->sign this extrinsic
  // + campaign_index: u32
  // + users: Vec<AccountId>
  // + amount: u128
  
  const users = ["5HMabVtSJsRrL2756NFeC269Bf5EmZm1zH21TvewmneaCZk5"]
  const unsub3 = await api.tx.sudo
  .sudo(
    api.tx.task.reward(0,users,10000 )
  )
  .signAndSend(ROOT, (result) => {
    if (result.status.isFinalized) {
      console.log(`Transaction finalized at blockHash ${result.status.asFinalized}`);
      console.log("Sudo key reward this campaign");
      unsub3();
    }

  });


  


}




main();