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

static void enqueue_sender(struct channel *ch, struct thread *t)
{
    t->next = NULL;
    if (ch->send_tail) ch->send_tail->next = t;
    else ch->send_head = t;
    ch->send_tail = t;
    t->blocked_on = ch;
}

static struct thread *dequeue_sender(struct channel *ch)
{
    struct thread *t = ch->send_head;
    if (!t) return NULL;
    ch->send_head = t->next;
    if (!ch->send_head) ch->send_tail = NULL;
    t->next = NULL;
    t->blocked_on = NULL;
    return t;
}

/* Copies the client's message into the server's buffer and returns the length stored. */
static long transfer_to_server(struct thread *server, struct thread *client, void *buf, size_t len)
{
    uint8_t bounce[MSG_MAX];
    size_t n = client->slen < len ? client->slen : len;
    if (n > MSG_MAX) n = MSG_MAX;
    if (copy_from_user(client->task, bounce, client->smsg, n)) return -EINVAL_;
    if (copy_to_user(server->task, buf, bounce, n)) return -EINVAL_;
    return (long)n;
}

int ipc_send(struct thread *client, int coid, const void *smsg, size_t slen, void *rmsg, size_t rlen)
{
    if (coid < 0 || coid >= MAX_CONNS || !conns[coid].chan) return -EBADF_;
    if (slen > MSG_MAX) return -EINVAL_;
    struct channel *ch = conns[coid].chan;
    client->smsg = smsg;
    client->slen = slen;
    client->rmsg = rmsg;
    client->rlen = rlen;
    struct thread *server = ch->receiver;
    if (server) {
        ch->receiver = NULL;
        long n = transfer_to_server(server, client, server->rbuf, server->rbuflen);
        if (n < 0) return (int)n;
        server->status = n;
        if (client->prio > server->prio) server->prio = client->prio;
        if (server->rcvid_out)
            *server->rcvid_out = client->id;
        client->server = server;
        client->state = T_REPLY;
        sched_ready(server);
        return 0;
    }
    client->state = T_SEND;
    enqueue_sender(ch, client);
    return 0;
}

long ipc_receive(struct thread *server, int chid, void *buf, size_t len, int *rcvid)
{
    if (chid < 0 || chid >= MAX_CHANNELS || !channels[chid].used) return -EBADF_;
    struct channel *ch = &channels[chid];
    struct thread *client = dequeue_sender(ch);
    if (!client) {
        server->rbuf = buf;
        server->rbuflen = len;
        server->rcvid_out = rcvid;
        ch->receiver = server;
        return 0;
    }
    long n = transfer_to_server(server, client, buf, len);
    if (n < 0) return n;
    if (client->prio > server->prio) server->prio = client->prio;
    client->server = server;
    client->state = T_REPLY;
    if (rcvid) *rcvid = client->id;
    return n;
}

int ipc_reply(struct thread *server, int rcvid, long status, const void *msg, size_t len)
{
    int id = rcvid;
    if (id <= 0 || id > MAX_THREADS) return -ESRCH_;
    struct thread *client = &threads[id - 1];
    if (client->state != T_REPLY || client->server != server) return -ESRCH_;
    if (len > client->rlen) len = client->rlen;
    if (len > MSG_MAX) len = MSG_MAX;
    if (len) {
        uint8_t bounce[MSG_MAX];
        if (copy_from_user(server->task, bounce, msg, len)) return -EINVAL_;
        if (copy_to_user(client->task, client->rmsg, bounce, len)) return -EINVAL_;
    }
    client->status = status;
    client->server = NULL;
    server->prio = server->base_prio;
    sched_ready(client);
    return 0;
}
