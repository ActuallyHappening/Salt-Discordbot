

MatchingEngine address: https://shannon-explorer.somnia.network/address/0x44E7525Cf9d56733D08fc98BcD750d504fCE91eC?tab=contract

cast decode-calldata "marketSellETH(address quote, bool isMaker, uint32 n, address recipient, uint32 slippageLimit)" 0x34c260100000000000000000000000000ed782b8079529f7385c3eda9faf1eaa0dbc6a1700000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000014000000000000000000000000385f8c5a2af2fbd503d55ab78d614bf0578ddbe00000000000000000000000000000000000000000000000000000000000989680

0x0ED782B8079529f7385c3eDA9fAf1EaA0DbC6a17
true
20
0x385f8c5A2AF2Fbd503D55AB78d614BF0578dDbe0
10000000 [1e7]

11:27:54 10 TRACE standard_sdk: Started tracing
11:27:54 10  INFO standard_sdk: My native SST balance pre sst="0.046209374000000000"
11:27:55 10  INFO standard_sdk: My USDC balance pre name="USD Coin" my_balance=28817
11:27:57 10  INFO standard_sdk: Bought some USDC but selling some of my SST: https://shannon-explorer.somnia.network/tx/0x40d3308b87d5dbfe8e5cfccab17296633b40e999b7cd6c15449bbb5ef36ed53d
11:27:58 10  INFO standard_sdk: My native SST balance post sst="0.028004290000000000"
11:27:58 10  INFO standard_sdk: My USDC balance post name="USD Coin" my_balance=28817

   function marketSellETH(address quote, bool isMaker, uint32 n, address recipient, uint32 slippageLimit) external payable returns (uint256 makePrice, uint256 placed, uint32 id);
    function mktPrice(address base, address quote) external view returns (uint256);

Call
cancelOrders(address[] base, address[] quote, bool[] isBid, uint32[] orderIds)
Name
Type
Data
base
address[]
[ "0x4A3BC48C156384f9564Fd65A53a2f3D534D8f2b7" ]
quote
address[]
[ "0x0ED782B8079529f7385c3eDA9fAf1EaA0DbC6a17" ]
isBid
bool[]
[ "false" ]
orderIds
uint32[]
[ "650" ]