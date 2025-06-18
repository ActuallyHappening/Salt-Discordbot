library ExchangeOrderbook {
    struct Order {
        address owner;
        uint256 price;
        uint256 depositAmount;
    }
}

library TransferHelper {
    struct TokenInfo {
        address token;
        uint8 decimals;
        string name;
        string symbol;
        uint256 totalSupply;
    }
}

interface MatchingEngine {
    error AccessControlBadConfirmation();
    error AccessControlUnauthorizedAccount(address account, bytes32 neededRole);
    error AlreadyInitialized(bool init);
    error AmountIsZero();
    error AskPriceTooHigh(uint256 limitPrice, uint256 lmp, uint256 maxAskPrice);
    error BidPriceTooLow(uint256 limitPrice, uint256 lmp, uint256 minBidPrice);
    error FactoryNotInitialized(address factory);
    error InvalidPair(address base, address quote, address pair);
    error InvalidRole(bytes32 role, address sender);
    error InvalidTerminal(address terminal);
    error NoLastMatchedPrice(address base, address quote);
    error NoOrderMade(address base, address quote);
    error OrderSizeTooSmall(uint256 amount, uint256 minRequired);
    error PairDoesNotExist(address base, address quote, address pair);
    error PairNotListedYet(address base, address quote, uint256 listingDate, uint256 timeNow);
    error ReentrancyGuardReentrantCall();
    error TooManyMatches(uint256 n);

    event ListingCostSet(address payment, uint256 amount);
    event NewMarketPrice(address pair, uint256 price, bool isBid);
    event OrderCanceled(address pair, uint256 id, bool isBid, address indexed owner, uint256 amount);
    event OrderDeposit(address sender, address asset, uint256 fee);
    event OrderMatched(address pair, uint256 id, bool isBid, address sender, address owner, uint256 price, uint256 amount, bool clear);
    event OrderPlaced(address pair, uint256 id, address owner, bool isBid, uint256 price, uint256 withoutFee, uint256 placed);
    event PairAdded(address pair, TransferHelper.TokenInfo base, TransferHelper.TokenInfo quote, uint256 listingPrice, uint256 listingDate, string supportedTerminals);
    event PairCreate2(address deployer, bytes bytecode);
    event PairUpdated(address pair, address base, address quote, uint256 listingPrice, uint256 listingDate);
    event RoleAdminChanged(bytes32 indexed role, bytes32 indexed previousAdminRole, bytes32 indexed newAdminRole);
    event RoleGranted(bytes32 indexed role, address indexed account, address indexed sender);
    event RoleRevoked(bytes32 indexed role, address indexed account, address indexed sender);

    constructor();

    receive() external payable;

    function DEFAULT_ADMIN_ROLE() external view returns (bytes32);
    function DENOM() external view returns (uint32);
    function WETH() external view returns (address);
    function addPair(address base, address quote, uint256 listingPrice, uint256 listingDate, address payment) external returns (address pair);
    function addPairETH(address base, address quote, uint256 listingPrice, uint256 listingDate) external payable returns (address book);
    function adjustPrice(address base, address quote, bool isBuy, uint256 price, uint256 assetAmount, uint32 beforeAdjust, uint32 afterAdjust, bool isMaker, uint32 n) external returns (uint256 makePrice, uint256 placed, uint32 id);
    function cancelOrder(address base, address quote, bool isBid, uint32 orderId) external returns (uint256);
    function cancelOrders(address[] memory base, address[] memory quote, bool[] memory isBid, uint32[] memory orderIds) external returns (uint256[] memory refunded);
    function convert(address base, address quote, uint256 amount, bool isBid) external view returns (uint256 converted);
    function getOrder(address base, address quote, bool isBid, uint32 orderId) external view returns (ExchangeOrderbook.Order memory);
    function getOrderIds(address base, address quote, bool isBid, uint256 price, uint32 n) external view returns (uint32[] memory);
    function getOrders(address base, address quote, bool isBid, uint256 price, uint32 n) external view returns (ExchangeOrderbook.Order[] memory);
    function getPair(address base, address quote) external view returns (address book);
    function getPrices(address base, address quote, bool isBid, uint32 n) external view returns (uint256[] memory);
    function getRoleAdmin(bytes32 role) external view returns (bytes32);
    function getSpread(address book, bool isBuy) external view returns (uint32 spreadLimit);
    function grantRole(bytes32 role, address account) external;
    function hasRole(bytes32 role, address account) external view returns (bool);
    function heads(address base, address quote) external view returns (uint256 bidHead, uint256 askHead);
    function initialize(address orderbookFactory_, address feeTo_, address WETH_) external;
    function limitBuy(address base, address quote, uint256 price, uint256 quoteAmount, bool isMaker, uint32 n, address recipient) external returns (uint256 makePrice, uint256 placed, uint32 id);
    function limitBuyETH(address base, uint256 price, bool isMaker, uint32 n, address recipient) external payable returns (uint256 makePrice, uint256 placed, uint32 id);
    function limitSell(address base, address quote, uint256 price, uint256 baseAmount, bool isMaker, uint32 n, address recipient) external returns (uint256 makePrice, uint256 placed, uint32 id);
    function limitSellETH(address quote, uint256 price, bool isMaker, uint32 n, address recipient) external payable returns (uint256 makePrice, uint256 placed, uint32 id);
    function listingDates(address) external view returns (uint256);
    function marketBuy(address base, address quote, uint256 quoteAmount, bool isMaker, uint32 n, address recipient, uint32 slippageLimit) external returns (uint256 makePrice, uint256 placed, uint32 id);
    function marketBuyETH(address base, bool isMaker, uint32 n, address recipient, uint32 slippageLimit) external payable returns (uint256 makePrice, uint256 placed, uint32 id);
    function marketSell(address base, address quote, uint256 baseAmount, bool isMaker, uint32 n, address recipient, uint32 slippageLimit) external returns (uint256 makePrice, uint256 placed, uint32 id);
    function marketSellETH(address quote, bool isMaker, uint32 n, address recipient, uint32 slippageLimit) external payable returns (uint256 makePrice, uint256 placed, uint32 id);
    function mktPrice(address base, address quote) external view returns (uint256);
    function orderbookFactory() external view returns (address);
    function rematchOrder(address base, address quote, bool isBid, uint32 orderId, uint256 price, uint256 amount, uint32 n) external returns (uint256 makePrice, uint256 placed, uint32 id);
    function renounceRole(bytes32 role, address callerConfirmation) external;
    function revokeRole(bytes32 role, address account) external;
    function setBaseFee(uint32 baseFee_) external returns (bool success);
    function setDefaultSpread(uint32 buy, uint32 sell) external returns (bool success);
    function setFeeTo(address feeTo_) external returns (bool success);
    function setIncentive(address incentive_) external returns (bool success);
    function setListingCost(string memory terminal, address payment, uint256 amount) external returns (uint256);
    function setMaxMatches(uint32 n) external returns (bool success);
    function setSpread(address base, address quote, uint32 buy, uint32 sell) external returns (bool success);
    function spreadLimits(address) external view returns (uint32 buy, uint32 sell);
    function supportsInterface(bytes4 interfaceId) external view returns (bool);
    function updatePair(address base, address quote, uint256 listingPrice, uint256 listingDate) external returns (address pair);
}