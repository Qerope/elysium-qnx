# elysium-qnx

[![ci](https://github.com/Qerope/elysium-qnx/actions/workflows/ci.yml/badge.svg)](https://github.com/Qerope/elysium-qnx/actions/workflows/ci.yml)
[![codeql](https://github.com/Qerope/elysium-qnx/actions/workflows/codeql.yml/badge.svg)](https://github.com/Qerope/elysium-qnx/actions/workflows/codeql.yml)

A RISC-V microkernel in the style of QNX Neutrino: the kernel does memory,
scheduling and synchronous message passing; everything else is a server.
Original code. The message API is QNX-*style* (`ChannelCreate`,
`ConnectAttach`, `MsgSend`, `MsgReceive`, `MsgReply`), not QNX source or
binary compatible.
