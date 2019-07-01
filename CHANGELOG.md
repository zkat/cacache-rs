<a name=""></a>
##  cacache changelog (2019-07-01)

Initial History generation.

#### Features

* **api:**  AsRef all the things! ([5af622eb](https://github.com/zkat/cacache-rs.git/commit/5af622eb30b9f177117ce2f8ad17690313fba50a))
* **content:**  add baseline read functionality ([e98bfb17](https://github.com/zkat/cacache-rs.git/commit/e98bfb17da0f4b862954e5f7636ea6284cd81367))
* **error:**
  *  Add SizeError ([0bbe080a](https://github.com/zkat/cacache-rs.git/commit/0bbe080a6ef636175ce07936ca8a7d26243509fb))
  *  add wrapper for atomicwrites ([dbb8c79b](https://github.com/zkat/cacache-rs.git/commit/dbb8c79b00f89e1b6303be179a6389328e1a762c))
* **errors:**  add errors module ([b0464849](https://github.com/zkat/cacache-rs.git/commit/b0464849e6cd32b047bbdfaa000e961dc2d87e86))
* **exports:**  re-export ssri::Algorithm and serde_json::Value ([87adc8cf](https://github.com/zkat/cacache-rs.git/commit/87adc8cf9f63211edc943e72ec28de797de574ea))
* **get:**
  *  add get::open() and get::open_hash() ([6e9a2f9f](https://github.com/zkat/cacache-rs.git/commit/6e9a2f9f87ecfb82a7bfd90fb748053a79de4e75))
  *  add external cacache::get api ([d91d2141](https://github.com/zkat/cacache-rs.git/commit/d91d2141761abf0e6180dc2ecd8c486637cf9232))
* **index:**
  *  make inserter.commit() return integrity ([257fc9b6](https://github.com/zkat/cacache-rs.git/commit/257fc9b6d0cb3f99547059821255b1719dd6be2f))
  *  implement delete() ([33a5dbbd](https://github.com/zkat/cacache-rs.git/commit/33a5dbbd51fc8d9ae180e8eac3f0600d8cbe37df))
  *  implemented find() ([44eb2acc](https://github.com/zkat/cacache-rs.git/commit/44eb2acc98b242747ff09460e0c276593dfe3840))
  *  implemented index::insert() ([322e68ff](https://github.com/zkat/cacache-rs.git/commit/322e68ffaa118ed519e1fe2f395b7cdfa903d91b))
  *  port index::insert() ([9ffc090b](https://github.com/zkat/cacache-rs.git/commit/9ffc090b3b2248def2aa9390ca1fd4028fb3663b))
* **ls:**  implemented cacache::ls::all() ([b0f351ea](https://github.com/zkat/cacache-rs.git/commit/b0f351ea269778e2e0be1d1388698d7a4b97ccd0))
* **path:**  ported content_path ([0f768fa5](https://github.com/zkat/cacache-rs.git/commit/0f768fa5c09445cc7dc81bcaea2639cf598f5107))
* **put:**
  *  privatize Put and PutOpts fields ([7f1602e2](https://github.com/zkat/cacache-rs.git/commit/7f1602e28fcecc02c47a43867c43dc8b420ca120))
  *  make PutOpts Clone ([27ce700b](https://github.com/zkat/cacache-rs.git/commit/27ce700bd69e1b72ab761521b0ba6fe0fc93ece1))
  *  Add put::Put and put::PutOpts ([15f017fe](https://github.com/zkat/cacache-rs.git/commit/15f017fe2151ad70dd75fbc90bae4c1cfccc00df))
  *  initial implementation of cacache::put ([815d7a3c](https://github.com/zkat/cacache-rs.git/commit/815d7a3c9e880eccd89baf4565e627658c5ac553))
* **read:**
  *  added has_content() ([bff95f20](https://github.com/zkat/cacache-rs.git/commit/bff95f20ec3f79a356a30733145f44adc99d2f83))
  *  added content read and read_to_string ([70cf52e1](https://github.com/zkat/cacache-rs.git/commit/70cf52e136624bbff415d2641d56331191649f17))
* **rm:**
  *  added external rm api ([346cf5fb](https://github.com/zkat/cacache-rs.git/commit/346cf5fb2379b9486186eca6aa14b72106818fc4))
  *  added content/rm ([eac29d94](https://github.com/zkat/cacache-rs.git/commit/eac29d941b0e36c143d3262e891fdbf991e316d7))
* **write:**  initial hack for write ([e452fdcd](https://github.com/zkat/cacache-rs.git/commit/e452fdcd16fae12d79602814979312767264a3b7))

#### Bug Fixes

* **api:**  use &str keys ([cf0fbe23](https://github.com/zkat/cacache-rs.git/commit/cf0fbe233f721f7ad3637eaf01207e3015f74ecd))
* **content:**  make rm use our own Error ([f3b6abf4](https://github.com/zkat/cacache-rs.git/commit/f3b6abf45c0408228e3bf8a0fe1e744d0b32c0bd))
* **fmt:**
  *  cargo fmt ([0349d115](https://github.com/zkat/cacache-rs.git/commit/0349d115f4e8d7aa59c6f7a0455b94be898efd46))
  *  cargo fmt ([bc56a1b3](https://github.com/zkat/cacache-rs.git/commit/bc56a1b3fee36f4ec2c3508ab34c3459904e1978))
* **index:**
  *  get rid of last compiler warning ([22c4b301](https://github.com/zkat/cacache-rs.git/commit/22c4b3010f9a851dd53073bbe1307ecbf01ef30e))
  *  make fields public, too ([65040481](https://github.com/zkat/cacache-rs.git/commit/6504048181415a4818fb6f713c7f9d7be665064a))
  *  switch to using new error module ([6f78e00c](https://github.com/zkat/cacache-rs.git/commit/6f78e00c42d59b73c725ebb4105983aee84459ff))
  *  make Entry use actual Integrity objects ([7ad0633c](https://github.com/zkat/cacache-rs.git/commit/7ad0633c4363a35a53e832dcac18b4672f462cc8))
  *  pass references instead of using .as_path() ([fc067e95](https://github.com/zkat/cacache-rs.git/commit/fc067e95d9c8dbb29ca1732e1e6bbd7b503239cc))
  *  remove unneeded integrity() method ([b579be61](https://github.com/zkat/cacache-rs.git/commit/b579be617f32a26ab557fb7944da89754e40c6ea))
* **lint:**  clippy told me to do this ([cba2f0d3](https://github.com/zkat/cacache-rs.git/commit/cba2f0d39afe71293742f97dcfd6c610031e5bfa))
* **put:**  fix warnings ([4a6950ff](https://github.com/zkat/cacache-rs.git/commit/4a6950ff5ddf6d3f110d2cf9bedeb1ef3134d1fa))
* **write:**  use shared Error type for write() ([8bf623b8](https://github.com/zkat/cacache-rs.git/commit/8bf623b8efab138f9a247edc45e477a08ab9213c))



