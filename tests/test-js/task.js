const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');
const BN = require('bn.js');

const main = async() => {
  const provider = new WsProvider('ws://18.139.30.168:9944');
  //const provider = new HttpProvider('http://localhost:9933');
  const api = await ApiPromise.create({ provider });

  


}




main();