#pragma once

#include <QApplication>

class App final : public QApplication {
public:
    App(int &argc, char **argv);
};
