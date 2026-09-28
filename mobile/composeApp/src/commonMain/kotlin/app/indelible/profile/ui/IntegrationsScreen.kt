package app.indelible.profile.ui

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import app.indelible.profile.ui.components.SettingsRow
import app.indelible.profile.ui.components.SettingsSection
import app.indelible.ui.theme.IndelibleSpacing
import indelible.composeapp.generated.resources.Res
import indelible.composeapp.generated.resources.common_back
import indelible.composeapp.generated.resources.integrations_add_library
import indelible.composeapp.generated.resources.integrations_add_library_description
import indelible.composeapp.generated.resources.integrations_add_rss_feed
import indelible.composeapp.generated.resources.integrations_add_rss_feed_description
import indelible.composeapp.generated.resources.integrations_content
import indelible.composeapp.generated.resources.integrations_manage_feeds
import indelible.composeapp.generated.resources.integrations_manage_feeds_description
import org.jetbrains.compose.resources.stringResource

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun IntegrationsScreen(
    onNavigateBack: () -> Unit,
    onNavigateToAddLibrary: () -> Unit,
    onNavigateToAddFeed: () -> Unit,
    onNavigateToFeeds: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Scaffold(
        modifier = modifier,
        topBar = {
            TopAppBar(
                title = {
                    Text(
                        text = stringResource(Res.string.integrations_content),
                        style = MaterialTheme.typography.headlineSmall,
                    )
                },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(
                            imageVector = Icons.AutoMirrored.Filled.ArrowBack,
                            contentDescription = stringResource(Res.string.common_back),
                        )
                    }
                },
            )
        },
    ) { paddingValues ->
        Column(
            modifier =
                Modifier
                    .fillMaxSize()
                    .padding(top = paddingValues.calculateTopPadding())
                    .verticalScroll(rememberScrollState()),
        ) {
            SettingsSection(title = stringResource(Res.string.integrations_content)) {
                SettingsRow(
                    label = stringResource(Res.string.integrations_add_library),
                    sublabel = stringResource(Res.string.integrations_add_library_description),
                    onClick = onNavigateToAddLibrary,
                )
                HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
                SettingsRow(
                    label = stringResource(Res.string.integrations_add_rss_feed),
                    sublabel = stringResource(Res.string.integrations_add_rss_feed_description),
                    onClick = onNavigateToAddFeed,
                )
                HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
                SettingsRow(
                    label = stringResource(Res.string.integrations_manage_feeds),
                    sublabel = stringResource(Res.string.integrations_manage_feeds_description),
                    onClick = onNavigateToFeeds,
                )
            }

            Spacer(modifier = Modifier.height(IndelibleSpacing.step32))
        }
    }
}
