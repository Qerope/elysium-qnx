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
