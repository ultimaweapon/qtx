#include "tab.hpp"

#include <new>

#include <stddef.h>

TabWidget::TabWidget(QWidget *parent) :
    QTabWidget(parent)
{
}

extern "C" TabWidget *qtx_tab_new(QWidget *parent, size_t size, size_t align)
{
    // Check alignment.
    if (align > __STDCPP_DEFAULT_NEW_ALIGNMENT__) {
        throw std::bad_alloc();
    }

    // Construct TabWidget.
    auto mem = operator new(size);

    try {
        return new(mem) TabWidget(parent);
    } catch (...) {
        operator delete(mem);
        throw;
    }
}

extern "C" {
    size_t qtx_tab_size = sizeof(TabWidget);
    size_t qtx_tab_align = alignof(TabWidget);
}
