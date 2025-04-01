'use server';

import { NextRequest, NextResponse } from 'next/server';

const slackHookURL = process.env.NEXT_SLACK_APP_SUPPORT_URL!;

export async function POST(request: NextRequest) {
  // Parse the request body
  const { userName, userEmail, organizationId, userId, message, subject, type } =
    await request.json();

  let slackMessage = {};

  if (type === 'feedback') {
    slackMessage = {
      text: 'New Support Request',
      blocks: [
        {
          type: 'section',
          text: {
            type: 'mrkdwn',
            text: '*New Support Request*'
          }
        },
        {
          type: 'section',
          fields: [
            {
              type: 'mrkdwn',
              text: `*Type:* ${type}`
            },
            {
              type: 'mrkdwn',
              text: `*From:* ${userName}`
            },
            {
              type: 'mrkdwn',
              text: `*Email:* ${userEmail}`
            },
            {
              type: 'mrkdwn',
              text: `*User ID:* ${userId}`
            },
            {
              type: 'mrkdwn',
              text: `*Organization ID:* ${organizationId}`
            }
          ]
        },
        {
          type: 'section',
          text: {
            type: 'mrkdwn',
            text: `*Message:*\n${message}`
          }
        }
      ]
    };
  } else {
    // Format the message for Slack
    slackMessage = {
      text: 'New Support Request',
      blocks: [
        {
          type: 'section',
          text: {
            type: 'mrkdwn',
            text: '*New Support Request*'
          }
        },
        {
          type: 'section',
          fields: [
            {
              type: 'mrkdwn',
              text: `*Type:* ${type}`
            },
            {
              type: 'mrkdwn',
              text: `*Subject:* ${subject}`
            },
            {
              type: 'mrkdwn',
              text: `*From:* ${userName}`
            },
            {
              type: 'mrkdwn',
              text: `*Email:* ${userEmail}`
            },
            {
              type: 'mrkdwn',
              text: `*User ID:* ${userId}`
            },
            {
              type: 'mrkdwn',
              text: `*Organization ID:* ${organizationId}`
            }
          ]
        },
        {
          type: 'section',
          text: {
            type: 'mrkdwn',
            text: `*Message:*\n${message}`
          }
        }
      ]
    };
  }

  // Send the formatted message to Slack
  const response = await fetch(slackHookURL, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify(slackMessage)
  });

  return NextResponse.json({
    success: response.ok
  });
}
