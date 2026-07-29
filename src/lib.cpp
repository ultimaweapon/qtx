#include <QApplication>
#include <QString>

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

extern "C" int qtx_application_exec()
{
    QGuiApplication::setQuitOnLastWindowClosed(false);

    return QApplication::exec();
}

extern "C" void qtx_exit(int code)
{
    QCoreApplication::exit(code);
}
