#include "main.hpp"

#include <qtx/string.hpp>

#include <QCloseEvent>

#include <stddef.h>
#include <stdint.h>

extern "C" {
    bool qtx_main_window_on_close(MainWindow *w);
    void qtx_main_window_on_window_title(MainWindow *w, const uint16_t *s, size_t l);
}

MainWindow::MainWindow()
{
    connect(this, &QWidget::windowTitleChanged, [this](const QString &v) {
        qtx_main_window_on_window_title(this, v.utf16(), v.size());
    });
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

extern "C" void qtx_main_window_destroy(MainWindow *w)
{
    w->~MainWindow();
}

extern "C" Str qtx_main_window_window_title(MainWindow *w)
{
    auto s = w->windowTitle();

    return qtx_str_from_utf16(s.utf16(), s.size());
}

extern "C" void qtx_main_window_set_window_title(MainWindow *w, const char *s, ptrdiff_t l)
{
    w->setWindowTitle(QString::fromUtf8(s, l));
}

extern "C" void qtx_main_window_set_central_widget(MainWindow *w, QWidget *v)
{
    w->setCentralWidget(v);
}

extern "C" void qtx_main_window_show(MainWindow *w)
{
    w->show();
}

extern "C" {
    size_t qtx_main_window_size = sizeof(MainWindow);
    size_t qtx_main_window_align = alignof(MainWindow);
}
