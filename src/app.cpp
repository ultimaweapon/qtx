#include "app.hpp"
#include "wake_task.hpp"

#include <new>

#include <stdint.h>

extern "C" {
    void qtx_app_poll_task(App *app, uint32_t id);
}

App::App(int &argc, char **argv) :
    QApplication(argc, argv)
{
}

bool App::event(QEvent *e)
{
    if (e->type() == WakeTask::Id) {
        qtx_app_poll_task(this, static_cast<WakeTask *>(e)->task());
        return true;
    }

    return QApplication::event(e);
}

extern "C" App *qtx_app_new(size_t size, size_t align, int *argc, char **argv)
{
    // Check alignment.
    if (align > __STDCPP_DEFAULT_NEW_ALIGNMENT__) {
        throw std::bad_alloc();
    }

    // Construct QApplication.
    auto mem = operator new(size);

    try {
        return new(mem) App(*argc, argv);
    } catch (...) {
        operator delete(mem);
        throw;
    }
}

extern "C" void qtx_app_destroy(App *app)
{
    delete app;
}

extern "C" void qtx_app_wake(uint32_t task)
{
    auto app = QCoreApplication::instance();

    QCoreApplication::postEvent(app, new WakeTask(task));
}

extern "C" {
    size_t qtx_app_size = sizeof(App);
    size_t qtx_app_align = alignof(App);
}
