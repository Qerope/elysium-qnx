#include <assert.h>
#include <stdio.h>
#include <string.h>

#include "ipc.h"

void sched_block(void) {}

static struct thread *mk(int id, int prio)
{
    struct thread *t = &threads[id - 1];
    memset(t, 0, sizeof *t);
    t->id = id;
    t->prio = t->base_prio = prio;
    return t;
}

int main(void)
{
    struct thread *cl = mk(1, 20), *srv = mk(2, 5);
    int chid = ChannelCreate_();
    int coid = ConnectAttach_(chid);
    assert(chid >= 0 && coid >= 0);

    char reply[16] = {0}, in[16] = {0};
    int rcvid = 0;

    /* client sends first: queued until the server receives */
    assert(ipc_send(cl, coid, "ping", 5, reply, sizeof reply) == 0);
    assert(cl->state == T_SEND);
    long n = ipc_receive(srv, chid, in, sizeof in, &rcvid);
    assert(n == 5 && strcmp(in, "ping") == 0);
    assert(cl->state == T_REPLY);
    assert(srv->prio == 20);   /* server runs at the client's priority while it works */
    assert(ipc_reply(srv, rcvid, 7, "pong", 5) == 0);
    assert(cl->status == 7 && strcmp(reply, "pong") == 0);
    assert(cl->state == T_READY);
    assert(srv->prio == 5);    /* and drops back afterwards */

    /* server receives first: blocked until a client sends */
    assert(ipc_receive(srv, chid, in, sizeof in, &rcvid) == 0);
    assert(ipc_send(cl, coid, "again", 6, reply, sizeof reply) == 0);
    assert(srv->status == 6 && strcmp(in, "again") == 0);
    assert(ipc_reply(srv, rcvid, 0, NULL, 0) == 0);
    assert(ipc_reply(srv, rcvid, 0, NULL, 0) == -ESRCH_);   /* second reply to the same rcvid */

    /* a stale receive id from an earlier message must not reach a newer one */
    assert(ipc_send(cl, coid, "one", 4, reply, sizeof reply) == 0);
    assert(ipc_receive(srv, chid, in, sizeof in, &rcvid) == 4);
    int old = rcvid;
    assert(ipc_reply(srv, old, 0, NULL, 0) == 0);
    assert(ipc_send(cl, coid, "two", 4, reply, sizeof reply) == 0);
    assert(ipc_receive(srv, chid, in, sizeof in, &rcvid) == 4);
    assert(ipc_reply(srv, old, 0, NULL, 0) == -ESRCH_);
    assert(ipc_reply(srv, rcvid, 0, NULL, 0) == 0);

    assert(ipc_send(cl, 99, "x", 2, reply, sizeof reply) == -EBADF_);
    printf("ipc ok\n");
    return 0;
}
