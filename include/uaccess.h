#pragma once
#include <stddef.h>

struct task;

/* Return 0 on success, -1 if any page in the range is unmapped or lacks user (and write) permission. */
int copy_from_user(struct task *t, void *kdst, const void *usrc, size_t n);
int copy_to_user(struct task *t, void *udst, const void *ksrc, size_t n);
