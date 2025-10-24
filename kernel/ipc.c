#include "ipc.h"

#include "kernel.h"
#include "uaccess.h"

#define MAX_CHANNELS 32
#define MAX_CONNS 64

struct channel {
    int used;
    struct thread *send_head, *send_tail;   /* clients waiting for a server to receive */
    struct thread *receiver;                /* a server blocked in MsgReceive */
};

struct conn {
    struct channel *chan;
};

static struct channel channels[MAX_CHANNELS];
static struct conn conns[MAX_CONNS];

int ChannelCreate_(void)
{
    for (int i = 0; i < MAX_CHANNELS; i++) {
        if (!channels[i].used) {
            channels[i] = (struct channel){.used = 1};
            return i;
        }
    }
    return -EINVAL_;
}

/* Connection ids are global for now; per-process tables come with the process manager. */
int ConnectAttach_(int chid)
{
    if (chid < 0 || chid >= MAX_CHANNELS || !channels[chid].used) return -EBADF_;
    for (int i = 0; i < MAX_CONNS; i++) {
        if (!conns[i].chan) {
            conns[i].chan = &channels[chid];
            return i;
        }
    }
    return -EINVAL_;
}
