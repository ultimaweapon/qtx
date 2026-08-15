#pragma once

#include <QMainWindow>

class MainWindow final : public QMainWindow {
public:
    MainWindow();
protected:
    void closeEvent(QCloseEvent *event) override;
};
