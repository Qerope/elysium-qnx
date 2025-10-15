#pragma once
#include <stddef.h>
#include <stdint.h>

struct ctx {
    uint64_t ra, sp, s[12];
};

enum tstate { T_READY, T_RUN, T_SEND, T_REPLY, T_RECV, T_DEAD };

#define MAX_THREADS 64
#define NPRI 32

struct task;
struct channel;

struct thread {
    struct ctx ctx;
    int id;
    int prio, base_prio;
    enum tstate state;
    struct thread *next;
    struct task *task;
};

extern struct thread threads[MAX_THREADS];
extern struct thread *current;

void ctx_switch(struct ctx *old, struct ctx *new_);
void sched_init(void);
struct thread *thread_create(void (*entry)(void), int prio);
void sched_ready(struct thread *t);
void sched_block(void);   /* current->state was set by the caller; returns once woken */
void schedule(void);
