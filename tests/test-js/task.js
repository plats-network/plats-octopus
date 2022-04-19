const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');
const BN = require('bn.js');

const main = async() => {
  const provider = new WsProvider('ws://18.139.30.168:9944');
  //const provider = new HttpProvider('http://localhost:9933');
  const api = await ApiPromise.create({ provider });


  // Check account balance
  const alice = '5Cfkwf5vfCF34SNbxLwmDrH97NTZ6KoztXiZov5wYxkYPY2u';
  const { nonce, data: balance } = await api.query.system.account(alice);
  console.log(`balance of ${balance.free} and a nonce of ${nonce}`);

  // create campaign
  const unsub = await api.tx.task
  .createCampaign(10000000000)
  .signAndSend(alice, (result) => {
    console.log(`Current status is ${result.status}`);

    if (result.status.isInBlock) {
      console.log(`Transaction included at blockHash ${result.status.asInBlock}`);
    } else if (result.status.isFinalized) {
      console.log(`Transaction finalized at blockHash ${result.status.asFinalized}`);
      unsub();
    }
  });

  


}




main();