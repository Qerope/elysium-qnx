#pragma once
#include <stddef.h>

#include "sched.h"

#define MSG_MAX 256
#define ESRCH_ 3
#define EINVAL_ 22
#define EBADF_ 9

int ChannelCreate_(void);
int ConnectAttach_(int chid);
/* Returns 0 if the client is now blocked waiting for the reply, or -errno. The reply status
   ends up in client->status after wakeup. */
int ipc_send(struct thread *client, int coid, const void *smsg, size_t slen, void *rmsg, size_t rlen);
/* Returns the message length if one was waiting, 0 if the server blocked (the result is
   delivered later in server->status), or -errno. */
long ipc_receive(struct thread *server, int chid, void *buf, size_t len, int *rcvid);
int ipc_reply(struct thread *server, int rcvid, long status, const void *msg, size_t len);
