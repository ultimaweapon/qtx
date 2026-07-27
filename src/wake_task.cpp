#include "wake_task.hpp"

const QEvent::Type WakeTask::Id = static_cast<QEvent::Type>(QEvent::registerEventType());

WakeTask::WakeTask(uint32_t task) :
    QEvent(Id),
    m_task(task)
{
}
