#include <QApplication>
#include <QString>

#include <memory>

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

extern "C" QApplication *qtx_application_new(int *argc, char **argv)
{
    auto app = std::make_unique<QApplication>(*argc, argv);

    return app.release();
}

extern "C" void qtx_application_destroy(QApplication *app)
{
    delete app;
}

extern "C" int qtx_application_exec()
{
    return QApplication::exec();
}
