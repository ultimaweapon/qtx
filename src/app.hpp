#pragma once

#include <QApplication>

class App final : public QApplication {
public:
    App(int &argc, char **argv);
protected:
    bool event(QEvent *e) override;
};
