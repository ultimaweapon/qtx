#include "executor.hpp"
#include "wake_task.hpp"

#include <QCoreApplication>

#include <stdint.h>

extern "C" {
    void qtx_executor_poll(Executor *exe, uint32_t id);
}

bool Executor::event(QEvent *event)
{
    if (event->type() == WakeTask::Id) {
        qtx_executor_poll(this, static_cast<WakeTask *>(event)->task());
        return true;
    }

    return QObject::event(event);
}

extern "C" Executor *qtx_executor_new(size_t size, size_t align)
{
    // Check alignment.
    if (align > __STDCPP_DEFAULT_NEW_ALIGNMENT__) {
        throw std::bad_alloc();
    }

    // Construct Executor.
    auto mem = operator new(size);

    try {
        return new(mem) Executor();
    } catch (...) {
        operator delete(mem);
        throw;
    }
}

extern "C" void qtx_executor_destroy(Executor *exe)
{
    exe->~Executor();
}

extern "C" void qtx_executor_wake(Executor *exe, uint32_t task)
{
    QCoreApplication::postEvent(exe, new WakeTask(task));
}

extern "C" {
    size_t qtx_executor_size = sizeof(Executor);
    size_t qtx_executor_align = alignof(Executor);
}
