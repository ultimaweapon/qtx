#include <QApplication>
#include <QString>

#include <new>

#include <stddef.h>

extern "C" bool qtx_application_set_style(const char *name, ptrdiff_t len)
{
    return QApplication::setStyle(QString::fromUtf8(name, len)) != nullptr;
}

extern "C" void qtx_application_set_organization_name(const char *name, ptrdiff_t len)
{
    QCoreApplication::setOrganizationName(QString::fromUtf8(name, len));
}

extern "C" void qtx_application_set_application_name(const char *name, ptrdiff_t len)
{
    QCoreApplication::setApplicationName(QString::fromUtf8(name, len));
}

extern "C" QApplication *qtx_application_new(size_t size, size_t align, int *argc, char **argv)
{
    // Check alignment.
    if (align > __STDCPP_DEFAULT_NEW_ALIGNMENT__) {
        throw std::bad_alloc();
    }

    // Construct QApplication.
    auto mem = operator new(size);

    try {
        return new(mem) QApplication(*argc, argv);
    } catch (...) {
        operator delete(mem);
        throw;
    }
}

extern "C" void qtx_application_destroy(QApplication *app)
{
    delete app;
}

extern "C" int qtx_application_exec()
{
    QGuiApplication::setQuitOnLastWindowClosed(false);

    return QApplication::exec();
}

extern "C" {
    size_t qtx_app_size = sizeof(QApplication);
    size_t qtx_app_align = alignof(QApplication);
}
