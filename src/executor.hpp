#pragma once

#include <QObject>

class Executor final : public QObject {
public:
    bool event(QEvent *event) override;
};
