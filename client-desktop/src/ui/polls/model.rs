pub fn compute_option_percentage(vote_count: u32, total_votes: u32) -> f32 {
    if total_votes == 0 {
        0.0
    } else {
        (vote_count as f32 / total_votes as f32).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sdk::protocol::{PollDto, PollOptionDto};
    use uuid::Uuid;

    #[test]
    fn test_compute_option_percentage() {
        assert_eq!(compute_option_percentage(0, 0), 0.0);
        assert_eq!(compute_option_percentage(5, 10), 0.5);
        assert_eq!(compute_option_percentage(10, 10), 1.0);
        assert_eq!(compute_option_percentage(1, 3), 1.0 / 3.0);
    }

    #[test]
    fn test_poll_options_percentage_distribution() {
        let poll = PollDto {
            id: Uuid::new_v4(),
            creator_id: Uuid::new_v4(),
            creator_name: "Host".to_string(),
            question: "Choose meeting time:".to_string(),
            options: vec![
                PollOptionDto {
                    id: 0,
                    text: "9:00 AM".to_string(),
                    vote_count: 2,
                    voter_ids: vec![],
                },
                PollOptionDto {
                    id: 1,
                    text: "1:00 PM".to_string(),
                    vote_count: 5,
                    voter_ids: vec![],
                },
                PollOptionDto {
                    id: 2,
                    text: "4:00 PM".to_string(),
                    vote_count: 3,
                    voter_ids: vec![],
                },
            ],
            multi_choice: false,
            is_anonymous: true,
            is_closed: false,
            total_votes: 10,
            created_at: "2026-08-14T20:00:00Z".to_string(),
        };

        let p0 = compute_option_percentage(poll.options[0].vote_count, poll.total_votes);
        let p1 = compute_option_percentage(poll.options[1].vote_count, poll.total_votes);
        let p2 = compute_option_percentage(poll.options[2].vote_count, poll.total_votes);

        assert_eq!(p0, 0.2);
        assert_eq!(p1, 0.5);
        assert_eq!(p2, 0.3);
        assert!((p0 + p1 + p2 - 1.0).abs() < 0.0001);
    }
}
