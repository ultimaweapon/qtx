#pragma once

#include <QMainWindow>

class MainWindow final : public QMainWindow {
protected:
    void closeEvent(QCloseEvent *event) override;
};
