#include <QApplication>

#include <memory>

extern "C" QApplication *qtx_application_new(int *argc, char **argv)
{
    auto app = std::make_unique<QApplication>(*argc, argv);

    return app.release();
}

extern "C" void qtx_application_destroy(QApplication *app)
{
    delete app;
}
