const { ApiPromise, WsProvider, Keyring } = require('@polkadot/api');
const BN = require('bn.js');

const main = async() => {
  const provider = new WsProvider('ws://18.139.30.168:9944');
  //const provider = new HttpProvider('http://localhost:9933');
  const api = await ApiPromise.create({ provider });

/*   const mnemonicAlice = mnemonicGenerate();
  console.log(`Generate mnemonic: ${mnemonicAlice}`);
  const isValidMnemonic = mnemonicValidate(mnemonicAlice);
  console.log(`isValidMnemonic: ${isValidMnemonic}`);
  const seedAlice = mnemonicToMiniSecret(mnemonicAlice);
  const { publicKey, secretKey } = naclKeypairFromSeed(seedAlice); */

  const nameTokenString = JSON.stringify(await api.rpc.system.properties());
  const nameToken = JSON.parse(nameTokenString).tokenSymbol[0]
  console.log(nameToken);
  //console.log(nameToken.toHuman().tokenSymbol[0]);


  // Check account balance
  const alice = '5CaYWkKW8hVBft5uuiQaKsatAnKPsbHBF2oSmQMZ5qAFZ9Et';
  const { nonce, data: balance } = await api.query.system.account(alice);

  console.log(`balance of ${balance.free} and a nonce of ${nonce}`);
  const decimals = api.registry.chainDecimals;
  const base = new BN(10).pow(new BN(decimals))
  //const mybal = ("${balance.free}");
  console.log(`BN Result *****:${balance.free}`);
  const dm = new BN(balance.free.toString()).divmod(base);
  console.log(parseFloat(dm.div.toString() + "." + dm.mod.toString()));



}




main();