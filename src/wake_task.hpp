#pragma once

#include <QEvent>

#include <stdint.h>

class WakeTask final : public QEvent {
public:
    static const Type Id;

    WakeTask(uint32_t task);

    uint32_t task() const noexcept { return m_task; }
private:
    uint32_t m_task;
};
