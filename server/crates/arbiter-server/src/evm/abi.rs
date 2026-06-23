use alloy::sol;

sol! {
    interface IERC20 {
        event Transfer(address indexed from, address indexed to, uint256 value);
        event Approval(address indexed owner, address indexed spender, uint256 value);

        function totalSupply() external view returns (uint256);
        function balanceOf(address account) external view returns (uint256);
        function transfer(address to, uint256 value) external returns (bool);
        function allowance(address owner, address spender) external view returns (uint256);
        function approve(address spender, uint256 value) external returns (bool);
        function transferFrom(address from, address to, uint256 value) external returns (bool);
    }
}

sol! {
    /// ERC-721: Non-Fungible Token Standard.
    #[derive(Debug)]
    interface IERC721 {
        event Transfer(address indexed from, address indexed to, uint256 indexed tokenId);
        event Approval(address indexed owner, address indexed approved, uint256 indexed tokenId);
        event ApprovalForAll(address indexed owner, address indexed operator, bool approved);

        function balanceOf(address owner) external view returns (uint256 balance);
        function ownerOf(uint256 tokenId) external view returns (address owner);
        function safeTransferFrom(address from, address to, uint256 tokenId) external;
        function safeTransferFrom(address from, address to, uint256 tokenId, bytes calldata data) external;
        function transferFrom(address from, address to, uint256 tokenId) external;
        function approve(address to, uint256 tokenId) external;
        function setApprovalForAll(address operator, bool approved) external;
        function getApproved(uint256 tokenId) external view returns (address operator);
        function isApprovedForAll(address owner, address operator) external view returns (bool);
    }
}

sol! {
    /// Wrapped Ether — the only functions beyond ERC-20 that matter.
    #[derive(Debug)]
    interface IWETH {
        function deposit() external payable;
        function withdraw(uint256 wad) external;
    }
}

sol! {
    /// Permit2 — Uniswap's canonical token approval manager.
    /// Replaces per-contract ERC-20 `approve()` with a single approval hub.
    #[derive(Debug)]
    interface IPermit2 {
        struct TokenPermissions {
            address token;
            uint256 amount;
        }

        struct PermitSingle {
            TokenPermissions details;
            address spender;
            uint256 sigDeadline;
        }

        struct PermitBatch {
            TokenPermissions[] details;
            address spender;
            uint256 sigDeadline;
        }

        struct AllowanceTransferDetails {
            address from;
            address to;
            uint160 amount;
            address token;
        }

        function approve(address token, address spender, uint160 amount, uint48 expiration) external;
        function permit(address owner, PermitSingle calldata permitSingle, bytes calldata signature) external;
        function permit(address owner, PermitBatch calldata permitBatch, bytes calldata signature) external;
        function transferFrom(address from, address to, uint160 amount, address token) external;
        function transferFrom(AllowanceTransferDetails[] calldata transferDetails) external;

        function allowance(address user, address token, address spender)
            external view returns (uint160 amount, uint48 expiration, uint48 nonce);
    }
}
