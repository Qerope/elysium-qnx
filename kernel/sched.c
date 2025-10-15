#include "sched.h"

#include "kernel.h"

struct thread threads[MAX_THREADS];
struct thread *current;

static struct thread *head[NPRI], *tail[NPRI];
static uint32_t bitmap;
static struct thread idle;

void sched_init(void)
{
    idle.state = T_RUN;
    current = &idle;
}

static void enqueue(struct thread *t)
{
    t->state = T_READY;
    t->next = NULL;
    if (tail[t->prio]) tail[t->prio]->next = t;
    else head[t->prio] = t;
    tail[t->prio] = t;
    bitmap |= 1U << t->prio;
}

struct thread *thread_create(void (*entry)(void), int prio)
{
    for (int i = 0; i < MAX_THREADS; i++) {
        struct thread *t = &threads[i];
        if (t->state != T_DEAD && t->id) continue;
        static char stacks[MAX_THREADS][4096] __attribute__((aligned(16)));
        t->id = i + 1;
        t->prio = t->base_prio = prio;
        t->ctx.ra = (uint64_t)(uintptr_t)entry;
        t->ctx.sp = (uint64_t)(uintptr_t)(stacks[i] + sizeof stacks[i]);
        sched_ready(t);
        return t;
    }
    return NULL;
}

void sched_ready(struct thread *t)
{
    unsigned long f = irq_save();
    enqueue(t);
    irq_restore(f);
}

void schedule(void)
{
    unsigned long f = irq_save();
    struct thread *prev = current;
    if (prev->state == T_RUN && prev != &idle) enqueue(prev);
    struct thread *next = &idle;
    if (bitmap) {
        int pri = 31 - __builtin_clz(bitmap);
        next = head[pri];
        head[pri] = next->next;
        if (!head[pri]) { tail[pri] = NULL; bitmap &= ~(1U << pri); }
    }
    next->state = T_RUN;
    if (next != prev) {
        current = next;
        ctx_switch(&prev->ctx, &next->ctx);
    }
    irq_restore(f);
}

void sched_block(void)
{
    schedule();
}
