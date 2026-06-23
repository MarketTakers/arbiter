#!/usr/bin/env python3
"""
Fetch the Uniswap default token list and emit Rust `TokenInfo` statics.

Usage:
    python3 gen_erc20_registry.py                       # fetch from IPFS
    python3 gen_erc20_registry.py tokens.json           # local file
    python3 gen_erc20_registry.py tokens.json out.rs    # custom output file
"""

import json
import re
import sys
import unicodedata
import urllib.request

UNISWAP_URL = "https://ipfs.io/ipns/tokens.uniswap.org"

SOLANA_CHAIN_ID = 501000101
IDENTIFIER_RE = re.compile(r"[^A-Za-z0-9]+")


def load_tokens(source=None):
    if source:
        with open(source) as f:
            return json.load(f)
    req = urllib.request.Request(
        UNISWAP_URL,
        headers={"Accept": "application/json", "User-Agent": "gen_tokens/1.0"},
    )
    with urllib.request.urlopen(req, timeout=60) as resp:
        return json.loads(resp.read())


def escape(s: str) -> str:
    return s.replace("\\", "\\\\").replace('"', '\\"')


def to_screaming_case(name: str) -> str:
    normalized = unicodedata.normalize("NFKD", name or "")
    ascii_name = normalized.encode("ascii", "ignore").decode("ascii")
    snake = IDENTIFIER_RE.sub("_", ascii_name).strip("_").upper()
    if not snake:
        snake = "TOKEN"
    if snake[0].isdigit():
        snake = f"TOKEN_{snake}"
    return snake


def static_name_for_token(token: dict, used_names: set[str]) -> str:
    base = to_screaming_case(token.get("name", ""))
    if base not in used_names:
        used_names.add(base)
        return base

    address = token["address"]
    suffix = f"{token['chainId']}_{address[2:].upper()[-8:]}"
    candidate = f"{base}_{suffix}"

    i = 2
    while candidate in used_names:
        candidate = f"{base}_{suffix}_{i}"
        i += 1

    used_names.add(candidate)
    return candidate


def main():
    source = sys.argv[1] if len(sys.argv) > 1 else None
    output = sys.argv[2] if len(sys.argv) > 2 else "generated_tokens.rs"
    data = load_tokens(source)
    tokens = data["tokens"]

    # Deduplicate by (chainId, address)
    seen = set()
    unique = []
    for t in tokens:
        key = (t["chainId"], t["address"].lower())
        if key not in seen:
            seen.add(key)
            unique.append(t)

    unique.sort(key=lambda t: (t["chainId"], t.get("symbol", "").upper()))
    evm_tokens = [t for t in unique if t["chainId"] != SOLANA_CHAIN_ID]

    ver = data["version"]
    lines = []
    w = lines.append

    w(
        f"// Auto-generated from Uniswap token list v{ver['major']}.{ver['minor']}.{ver['patch']}"
    )
    w(f"// {len(evm_tokens)} tokens")
    w("// DO NOT EDIT - regenerate with gen_erc20_registry.py")
    w("")

    used_static_names = set()
    token_statics = []
    for t in evm_tokens:
        static_name = static_name_for_token(t, used_static_names)
        token_statics.append((static_name, t))

    for static_name, t in token_statics:
        addr = t["address"]
        name = escape(t.get("name", ""))
        symbol = escape(t.get("symbol", ""))
        decimals = t.get("decimals", 18)
        logo = t.get("logoURI")
        chain = t["chainId"]

        logo_val = f'Some("{escape(logo)}")' if logo else "None"

        w(f"pub static {static_name}: TokenInfo = TokenInfo {{")
        w(f'    name: "{name}",')
        w(f'    symbol: "{symbol}",')
        w(f"    decimals: {decimals},")
        w(f'    contract: address!("{addr}"),')
        w(f"    chain: {chain},")
        w(f"    logo_uri: {logo_val},")
        w("};")
        w("")

    w("pub static TOKENS: &[&TokenInfo] = &[")
    for static_name, _ in token_statics:
        w(f"    &{static_name},")
    w("];")
    w("")
    w("pub fn get_token(")
    w("    chain_id: alloy::primitives::ChainId,")
    w("    address: alloy::primitives::Address,")
    w(") -> Option<&'static TokenInfo> {")
    w("    match (chain_id, address) {")
    for static_name, t in token_statics:
        w(
            f'        ({t["chainId"]}, addr) if addr == address!("{t["address"]}") => Some(&{static_name}),'
        )
    w("        _ => None,")
    w("    }")
    w("}")
    w("")

    with open(output, "w") as f:
        f.write("\n".join(lines))

    print(f"Wrote {len(token_statics)} tokens to {output}")


if __name__ == "__main__":
    main()
