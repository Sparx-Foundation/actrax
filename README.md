# Actrax

Rust Command and Control Server

set log level with env var `ACTRAX_LOG_LEVEL` (Trace, Debug, Info, Warn, Error)
for more info check out:
https://docs-rs-web-prod.infra.rust-lang.org/env_logger/latest/env_logger/index.html#enabling-logging

and

`ACTRAX_LOG_STYLE` for logging read more:

https://docs-rs-web-prod.infra.rust-lang.org/env_logger/latest/env_logger/index.html#disabling-colors

## TODO

- [] multiple endpoints (grpc, and others...)

## Install

use cargo:

```shell
cargo run --release
```

or simply run it with podman:

```shell
podman build -t actrax_server .
podman run --network actrax --name actrax_server -e "ACTRAX_SERVER_LOG=trace" -p 4444:4444 --replace actrax_server
```

I tested it with podman you can use docker as well. (maybe)

## Modules

[//]: # (TODO: add wiki page about modules)
read more about modules in the wiki (soon)


## Disclaimer

> [!CAUTION]  
> This software is provided for educational purposes only. It is intended to be used in accordance with all applicable
> laws and regulations. The authors and contributors are not responsible for any misuse, illegal activities, or damage
> caused by the use of this software. Users are solely responsible for ensuring that their use of this software complies
> with all local, state, and federal laws.

## License

This project is licensed under the GNU Affero General Public License v3.0 - see the [LICENSE](LICENSE-AGPLv3) file for details.

You should have received a copy of the GNU General Public License along with this program.
If not, see https://www.gnu.org/licenses/agpl-3.0.txt.


> [!TIP]
> ### TL;DR
> 
>  The AGPLv3 ensures that any modifications or uses of AGPLv3-licensed software,  
> including the running of such software on servers or the distribution of it,  
> must be accompanied by the sharing of the modified source code with others under the same licence terms.