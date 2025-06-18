library IOrderbookFactory {
    struct Pair {
        address base;
        address quote;
    }
}

interface OrderbookFactory {
    error InvalidAccess(address sender, address allowed);
    error InvalidInitialization();
    error NotInitializing();
    error PairAlreadyExists(address base, address quote, address pair);
    error SameBaseQuote(address base, address quote);

    event Initialized(uint64 version);

    constructor();

    function allPairs(uint256) external view returns (address);
    function allPairsLength() external view returns (uint256);
    function createBook(address base_, address quote_) external returns (address orderbook);
    function engine() external view returns (address);
    function getBaseQuote(address orderbook) external view returns (address base, address quote);
    function getByteCode() external view returns (bytes memory bytecode);
    function getListingCost(string memory terminal, address payment) external view returns (uint256);
    function getPair(address base, address quote) external view returns (address book);
    function getPairNames(uint256 start, uint256 end) external view returns (string[] memory names);
    function getPairNamesWithIds(uint256[] memory ids) external view returns (string[] memory names);
    function getPairs(uint256 start, uint256 end) external view returns (IOrderbookFactory.Pair[] memory);
    function getPairsWithIds(uint256[] memory ids) external view returns (IOrderbookFactory.Pair[] memory pairs);
    function impl() external view returns (address);
    function initialize(address engine_) external returns (address);
    function isClone(address vault) external view returns (bool cloned);
    function listingCosts(string memory, address) external view returns (uint256);
    function setListingCost(string memory terminal, address payment, uint256 amount) external returns (uint256);
    function version() external view returns (uint32);
}