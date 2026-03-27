use super::*;
use super::obsidian::ObsidianAdapter;
use super::markdown::MarkdownAdapter;
use super::dayone::DayOneAdapter;
use super::facebook::FacebookAdapter;
use super::instagram::InstagramAdapter;
use super::twitter::TwitterAdapter;
use super::reddit::RedditAdapter;
use super::linkedin::LinkedInAdapter;
use super::google_takeout::GoogleTakeoutAdapter;
use super::whatsapp::WhatsAppAdapter;
use super::telegram::TelegramAdapter;
use super::discord::DiscordAdapter;
use super::snapchat::SnapchatAdapter;
use super::tiktok::TikTokAdapter;
use super::youtube::YouTubeAdapter;
use super::spotify::SpotifyAdapter;
use super::slack::SlackAdapter;
use super::evernote::EvernoteAdapter;
use super::bluesky::BlueskyAdapter;
use super::notion::NotionAdapter;
use super::signal::SignalAdapter;
use super::mastodon::MastodonAdapter;
use super::threads::ThreadsAdapter;
use super::substack::SubstackAdapter;
use super::medium::MediumAdapter;
use super::tumblr::TumblrAdapter;
use super::pinterest::PinterestAdapter;
use super::apple::AppleAdapter;
use super::amazon::AmazonAdapter;
use super::netflix::NetflixAdapter;
use super::microsoft::MicrosoftAdapter;
use super::browser_history::{ChromeHistoryAdapter, EdgeHistoryAdapter, FirefoxHistoryAdapter, SafariHistoryAdapter};
use super::generic::GenericAdapter;

/// Returns all registered source adapters.
pub fn all_adapters() -> Vec<Box<dyn SourceAdapter>> {
    vec![
        Box::new(ObsidianAdapter),
        Box::new(MarkdownAdapter),
        Box::new(DayOneAdapter),
        Box::new(FacebookAdapter),
        Box::new(InstagramAdapter),
        Box::new(TwitterAdapter),
        Box::new(RedditAdapter),
        Box::new(LinkedInAdapter),
        Box::new(GoogleTakeoutAdapter),
        Box::new(WhatsAppAdapter),
        Box::new(TelegramAdapter),
        Box::new(DiscordAdapter),
        Box::new(SnapchatAdapter),
        Box::new(TikTokAdapter),
        Box::new(YouTubeAdapter),
        Box::new(SpotifyAdapter),
        Box::new(SlackAdapter),
        Box::new(EvernoteAdapter),
        Box::new(BlueskyAdapter),
        Box::new(NotionAdapter),
        Box::new(SignalAdapter),
        Box::new(MastodonAdapter),
        Box::new(ThreadsAdapter),
        Box::new(SubstackAdapter),
        Box::new(MediumAdapter),
        Box::new(TumblrAdapter),
        Box::new(PinterestAdapter),
        Box::new(AppleAdapter),
        Box::new(AmazonAdapter),
        Box::new(NetflixAdapter),
        Box::new(MicrosoftAdapter),
        // Browser history
        Box::new(ChromeHistoryAdapter),
        Box::new(EdgeHistoryAdapter),
        Box::new(FirefoxHistoryAdapter),
        Box::new(SafariHistoryAdapter),
        Box::new(GenericAdapter), // must be last (fallback)
    ]
}

/// Returns all adapter metadata (for frontend listing).
pub fn all_adapter_metadata() -> Vec<SourceAdapterMeta> {
    all_adapters().iter().map(|a| a.metadata()).collect()
}

/// Auto-detect the best adapter for a given file listing.
pub fn detect_adapter(file_listing: &[&str]) -> Option<Box<dyn SourceAdapter>> {
    let mut best: Option<(f32, Box<dyn SourceAdapter>)> = None;
    for adapter in all_adapters() {
        let score = adapter.detect(file_listing);
        if score > best.as_ref().map(|(s, _)| *s).unwrap_or(0.0) {
            best = Some((score, adapter));
        }
    }
    best.map(|(_, a)| a)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_adapters_not_empty() {
        let adapters = all_adapters();
        assert!(adapters.len() >= 30);
    }

    #[test]
    fn test_generic_is_last() {
        let adapters = all_adapters();
        let last = adapters.last().unwrap();
        assert_eq!(last.name(), "generic");
    }

    #[test]
    fn test_all_adapter_metadata() {
        let metas = all_adapter_metadata();
        assert!(metas.len() >= 30);
        // Each should have a non-empty id
        for m in &metas {
            assert!(!m.id.is_empty());
        }
    }

    #[test]
    fn test_detect_adapter_fallback() {
        // With unknown files, the generic adapter (0.1) should still win
        let result = detect_adapter(&["unknown_file.xyz"]);
        assert!(result.is_some());
        assert_eq!(result.unwrap().name(), "generic");
    }

    #[test]
    fn test_detect_adapter_specific() {
        let result = detect_adapter(&["messages/inbox/alice/message_1.json"]);
        assert!(result.is_some());
        assert_eq!(result.unwrap().name(), "facebook");
    }
}
