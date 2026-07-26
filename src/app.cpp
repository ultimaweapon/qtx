#include "app.hpp"

#include <new>

App::App(int &argc, char **argv)
    : QApplication(argc, argv)
{
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

extern "C" {
    size_t qtx_app_size = sizeof(App);
    size_t qtx_app_align = alignof(App);
}
