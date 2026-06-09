# Arbiter
> Policy-first multi-client wallet daemon, allowing permissioned transactions across blockchains

## Security warning
Arbiter can't meaningfully protect against host compromise. Potential attack flow:
- Attacker steals TLS keys from database
- Pretends to be server; just accepts operator challenge solutions
- Pretend to be in sealed state and performing DH with client
- Steals user password and derives seal key

While this attack is highly targetive, it's still possible. 

> This software is experimental. Do not use with funds you cannot afford to lose.