#include "main.hpp"

#include <QCloseEvent>

extern "C" {
    bool qtx_main_window_on_close(MainWindow *win);
}

void MainWindow::closeEvent(QCloseEvent *event)
{
    // This will set to accept by QMainWindow::closeEvent.
    event->ignore();

    if (qtx_main_window_on_close(this)) {
        QMainWindow::closeEvent(event);
    }
}

extern "C" MainWindow *qtx_main_window_new(size_t size, size_t align)
{
    // Check alignment.
    if (align > __STDCPP_DEFAULT_NEW_ALIGNMENT__) {
        throw std::bad_alloc();
    }

    // Construct QApplication.
    auto mem = operator new(size);

    try {
        return new(mem) MainWindow();
    } catch (...) {
        operator delete(mem);
        throw;
    }
}

extern "C" void qtx_main_window_destroy(MainWindow *win)
{
    win->~MainWindow();
}

extern "C" void qtx_main_window_show(MainWindow *win)
{
    win->show();
}

extern "C" {
    size_t qtx_main_window_size = sizeof(MainWindow);
    size_t qtx_main_window_align = alignof(MainWindow);
}
