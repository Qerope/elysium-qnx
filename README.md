# elysium-qnx

A RISC-V microkernel in the style of QNX Neutrino: the kernel does memory,
scheduling and synchronous message passing; everything else is a server.
Original code. The message API is QNX-*style* (`ChannelCreate`,
`ConnectAttach`, `MsgSend`, `MsgReceive`, `MsgReply`), not QNX source or
binary compatible.
