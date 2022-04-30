// SPDX-License-Identifier: MIT
pragma solidity >=0.8.0 <0.9.0;

import "@openzeppelin/contracts/access/Ownable.sol";
import "./interfaces/IPriceFeed.sol";
import "./interfaces/IMint.sol";
import "./sAsset.sol";
import "./EUSD.sol";

contract Mint is Ownable, IMint{

    struct Asset {
        address token;
        uint minCollateralRatio;
        address priceFeed;
    }

    struct Position {
        uint idx;
        address owner;
        uint collateralAmount;
        address assetToken;
        uint assetAmount;
    }

    mapping(address => Asset) _assetMap;
    uint _currentPositionIndex;
    mapping(uint => Position) _idxPositionMap;
    address public collateralToken;
    

    constructor(address collateral) {
        collateralToken = collateral;
    }

    function registerAsset(address assetToken, uint minCollateralRatio, address priceFeed) external override onlyOwner {
        require(assetToken != address(0), "Invalid assetToken address");
        require(minCollateralRatio >= 1, "minCollateralRatio must be greater than 100%");
        require(_assetMap[assetToken].token == address(0), "Asset was already registered");
        
        _assetMap[assetToken] = Asset(assetToken, minCollateralRatio, priceFeed);
    }

    function getPosition(uint positionIndex) external view returns (address, uint, address, uint) {
        require(positionIndex < _currentPositionIndex, "Invalid index");
        Position storage position = _idxPositionMap[positionIndex];
        return (position.owner, position.collateralAmount, position.assetToken, position.assetAmount);
    }

    function getMintAmount(uint collateralAmount, address assetToken, uint collateralRatio) public view returns (uint) {
        Asset storage asset = _assetMap[assetToken];
        (int relativeAssetPrice, ) = IPriceFeed(asset.priceFeed).getLatestPrice();
        uint8 decimal = sAsset(assetToken).decimals();
        uint mintAmount = collateralAmount * (10 ** uint256(decimal)) / uint(relativeAssetPrice) / collateralRatio ;
        return mintAmount;
    }

    function checkRegistered(address assetToken) public view returns (bool) {
        return _assetMap[assetToken].token == assetToken;
    }

    function openPosition(uint collateralAmount, address assetToken, uint collateralRatio) external override onlyOwner{
        require(checkRegistered(assetToken), "asset is registered");
        Asset storage asset = _assetMap[assetToken];

        require(collateralRatio >= asset.minCollateralRatio,"must be greater than MCR");
        uint assetAmount = getMintAmount(collateralAmount, assetToken, collateralRatio);

        _idxPositionMap[_currentPositionIndex] = Position(_currentPositionIndex, msg.sender, collateralAmount, assetToken, assetAmount);
        _currentPositionIndex = _currentPositionIndex + 1;

        EUSD(collateralToken).transferFrom(msg.sender, address(this), collateralAmount);
        sAsset(assetToken).mint(msg.sender, assetAmount);
    }

    function closePosition(uint positionIndex) external override onlyOwner{
        require(positionIndex < _currentPositionIndex, "Invalid index");
        require(_idxPositionMap[positionIndex].collateralAmount > 0," This position has been deleted");
        Position storage position = _idxPositionMap[positionIndex];
        EUSD(collateralToken).transfer(msg.sender,position.collateralAmount);
        sAsset(position.assetToken).burn(msg.sender, position.assetAmount);
        delete _idxPositionMap[position.idx];
    }

    function deposit(uint positionIndex, uint collateralAmount) external override onlyOwner{
        require(positionIndex < _currentPositionIndex, "Invalid index");
        require(_idxPositionMap[positionIndex].collateralAmount > 0," This position has been deleted");
        _idxPositionMap[positionIndex].collateralAmount += collateralAmount;
        EUSD(collateralToken).transferFrom(msg.sender,address(this), collateralAmount);
    }

    function withdraw(uint positionIndex, uint withdrawAmount) external override onlyOwner{
        Position storage position = _idxPositionMap[positionIndex];
        require(positionIndex < _currentPositionIndex, "Invalid index");
        require(_idxPositionMap[positionIndex].collateralAmount > 0," This position has been deleted");

        uint temp = _idxPositionMap[positionIndex].collateralAmount;
        uint asset_amount = _idxPositionMap[positionIndex].assetAmount;
        Asset storage asset = _assetMap[_idxPositionMap[positionIndex].assetToken];
        require(withdrawAmount <= temp, "withdraw should smaller than balance");
        temp = temp - withdrawAmount;
        uint MCR = asset.minCollateralRatio;
        address token = position.assetToken;
        (int relativeAssetPrice, ) = IPriceFeed(asset.priceFeed).getLatestPrice();
        uint8 decimal = sAsset(token).decimals();
        require((temp*(10**uint256(decimal))) / (position.assetAmount*uint(relativeAssetPrice)) >= MCR, "not meet the requirement of bigger then MCR");
        _idxPositionMap[positionIndex].collateralAmount = temp;
        EUSD(collateralToken).transfer(msg.sender,withdrawAmount);
    }

    function mint(uint positionIndex, uint mintAmount) external override onlyOwner{
        require(positionIndex < _currentPositionIndex, "Invalid index");
        require(_idxPositionMap[positionIndex].collateralAmount > 0," This position has been deleted");
        Position storage position = _idxPositionMap[positionIndex];
        uint asset_amount = position.assetAmount+ mintAmount;
        uint coll_amount = position.collateralAmount;
        Asset storage asset = _assetMap[position.assetToken];
        address token = position.assetToken;
        uint MCR = asset.minCollateralRatio;
        (int relativeAssetPrice, ) = IPriceFeed(asset.priceFeed).getLatestPrice();
        uint8 decimal = sAsset(token).decimals();

        require((position.collateralAmount*(10**uint256(decimal))) / (asset_amount*uint(relativeAssetPrice)) >= MCR, "not meet the requirement of bigger then MCR");
        position.assetAmount = asset_amount;
        sAsset(position.assetToken).mint(msg.sender, mintAmount);
    }

    function burn(uint positionIndex, uint burnAmount) external override onlyOwner{
        require(positionIndex < _currentPositionIndex, "Invalid index");
        require(_idxPositionMap[positionIndex].collateralAmount > 0," This position has been deleted");
        Position storage position = _idxPositionMap[positionIndex];
        require(burnAmount <= position.assetAmount,"not enough");
        uint asset_amount = position.assetAmount - burnAmount;
        position.assetAmount = asset_amount;
        sAsset(position.assetToken).burn(msg.sender, burnAmount);
    }
}