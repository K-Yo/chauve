use crate::entities::alert_list::AlertList;


pub trait PullAlerts {
    fn pull(&self) -> AlertList;
}
