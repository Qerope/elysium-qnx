#pragma once
#include <stddef.h>

#include "sched.h"

#define MSG_MAX 256
#define ESRCH_ 3
#define EINVAL_ 22
#define EBADF_ 9

int ChannelCreate_(void);
int ConnectAttach_(int chid);
