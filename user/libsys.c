#include <stddef.h>

#include "sys.h"

static inline long ecall(long n, long a, long b, long c, long d, long e)
{
    register long a0 __asm__("a0") = a;
    register long a1 __asm__("a1") = b;
    register long a2 __asm__("a2") = c;
    register long a3 __asm__("a3") = d;
    register long a4 __asm__("a4") = e;
    register long a7 __asm__("a7") = n;
    __asm__ volatile("ecall" : "+r"(a0) : "r"(a1), "r"(a2), "r"(a3), "r"(a4), "r"(a7) : "memory");
    return a0;
}

int ChannelCreate(void) { return (int)ecall(SYS_ChannelCreate, 0, 0, 0, 0, 0); }
int ConnectAttach(int chid) { return (int)ecall(SYS_ConnectAttach, chid, 0, 0, 0, 0); }

long MsgSend(int coid, const void *smsg, size_t slen, void *rmsg, size_t rlen)
{
    return ecall(SYS_MsgSend, coid, (long)smsg, (long)slen, (long)rmsg, (long)rlen);
}

long MsgReceive(int chid, void *buf, size_t len, int *rcvid)
{
    return ecall(SYS_MsgReceive, chid, (long)buf, (long)len, (long)rcvid, 0);
}

int MsgReply(int rcvid, long status, const void *msg, size_t len)
{
    return (int)ecall(SYS_MsgReply, rcvid, status, (long)msg, (long)len, 0);
}
